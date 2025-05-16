## Part 6: Advanced Topics

### 6.1 In-depth Examination of Lifetimes and GC Interaction

-   The `'gc` lifetime parameter on [`OCaml<'gc, T>`](OCaml) is of critical importance. It is bound to the
    scope wherein an [`OCamlRuntime`] is active and the domain lock is held
    (e.g., within [`OCamlRuntime::with_domain_lock`](OCamlRuntime::with_domain_lock) or the body of an
    [`#[ocaml_interop::export]`](export)-annotated function).
-   An [`OCaml<'gc, T>`](OCaml) value is guaranteed to be valid only for the duration of that `'gc`
    lifetime. If any invocation into OCaml occurs (which could potentially trigger the GC),
    **or if the OCaml runtime is re-entered through any means (e.g., a nested
    [`OCamlRuntime::with_domain_lock`] call, or a callback from OCaml back into Rust that then calls OCaml
    again)**, an unrooted [`OCaml<T>`](OCaml) value may become stale (i.e., a dangling pointer). This is
    because such re-entrancy can lead to GC cycles or other runtime operations that invalidate
    previous, unrooted handles.
-   **It is imperative to root OCaml values using [`BoxRoot<T>`](BoxRoot) if they need to be preserved
    across OCaml calls, runtime re-entrancy, or Rust scopes where the original `'gc` lifetime
    does not apply.**

### 6.2 [`OCamlRef<'a, T>`](OCamlRef) Detailed Explanation

-   [`OCamlRef<'a, T>`](OCamlRef) constitutes a lightweight reference to an OCaml value. The lifetime `'a` is
    typically the lifetime of a borrowed [`OCaml<T>`](OCaml) or [`BoxRoot<T>`](BoxRoot).
-   Its primary application is for passing arguments to OCaml functions declared with the [`ocaml!`]
    macro.
-   It does not, in itself, provide rooting; the value to which it refers must already be valid
    and appropriately rooted for the duration of the call.
-   [`OCamlRef<T>`](OCamlRef) instances possess a `.to_rust(cr)` method for conversion,
    which requires an explicit [`OCamlRuntime`] reference.

### 6.3 Interacting with OCaml Closures

[Content to be added later. This section will explain how to work with OCaml closures.]

### 6.4 Tuples

OCaml tuples can be seamlessly converted to and from Rust tuples. The `ocaml-interop` crate
provides implementations of [`ToOCaml<T>`](ToOCaml) and [`FromOCaml<T>`](FromOCaml) for tuples up to a certain arity
(currently 9 elements), mapping OCaml's `*` types to Rust's `()` tuple types.

**Key Concepts:**

*   **Representation:** An OCaml tuple like `int * string` is represented in Rust as
    `OCaml<(OCamlInt, String)>` when received as an argument or an unrooted value.
*   **Conversion from OCaml to Rust:**
    *   You can convert an entire `OCaml<(OCamlTypeA, OCamlTypeB, ...)>` to a Rust tuple
        `(RustTypeA, RustTypeB, ...)` using the `.to_rust()` method. This is generally the most
        direct way.
    *   Individual elements of an `OCaml<(OCamlTypeA, OCamlTypeB)>` can be accessed as
        `OCaml<OCamlTypeA>` and `OCaml<OCamlTypeB>` using `.fst()` and `.snd()` methods
        respectively (with `.tuple_3()`, `.tuple_4()`, etc., for larger tuples if those
        accessors are defined on the `OCaml<T>` type for tuples). These individual [`OCaml<T>`](OCaml)
        elements can then be converted to their Rust counterparts using `.to_rust()`.
*   **Conversion from Rust to OCaml:**
    *   A Rust tuple `(RustTypeA, RustTypeB, ...)` can be converted to an
        `OCaml<(OCamlTypeA, OCamlTypeB, ...)>` using `.to_ocaml(cr)` or, more commonly for
        return values, to a `BoxRoot<(OCamlTypeA, OCamlTypeB, ...)>` using `.to_boxroot(cr)`.

**Example:**

For a complete buildable example demonstrating OCaml-Rust tuple interoperability, please see the `docs/examples/tuples/` directory.

### 6.5 Records

OCaml records can be seamlessly converted to and from Rust structs using the
[`impl_conv_ocaml_record!`] macro provided by `ocaml-interop`. This macro automates the
generation of [`ToOCaml`] and [`FromOCaml`] trait implementations for your Rust struct, handling the
field-by-field mapping.

**Key Concepts:**

*   **Rust Struct Definition:** Define a plain Rust struct that mirrors the OCaml record's
    structure.
*   **[`impl_conv_ocaml_record!`] Macro:** This macro is the primary tool for enabling
    bidirectional conversion. You specify your Rust struct and then list its fields, mapping each
    to its corresponding OCaml type (e.g., Rust `String` to OCaml `string`, Rust `i64` to OCaml
    `int` which is represented as `OCamlInt` at the interop layer).
    *   This macro effectively combines the functionality of [`impl_to_ocaml_record!`] (for Rust to
        OCaml conversion) and [`impl_from_ocaml_record!`] (for OCaml to Rust conversion). If you
        only need one-way conversion, you can use these more specific macros directly.
    *   **Crucially, the order of fields declared within the macro must exactly match the order
        of fields in the OCaml record definition.**
    *   The macro also supports specifying different names if the Rust struct name and the Rust
        marker type for the OCaml record differ, using the
        `RustStructName => OCamlMarkerTypeName { ... }` syntax. For example, if your Rust struct
        is `MyPersonStruct` and the Rust marker type representing the OCaml record (which might
        be named `person` in OCaml) is `OCamlPersonRecord`, you would use
        `MyPersonStruct => OCamlPersonRecord { ... }`. If this syntax is not used, the macro
        assumes the Rust struct name also serves as the marker type name (e.g., `Person` implies
        `OCaml<Person>`).
*   **Conversion Methods:**
    *   **From OCaml to Rust:** Once the appropriate [`FromOCaml<T>`](FromOCaml) trait is implemented (either via
        [`impl_conv_ocaml_record!`] or [`impl_from_ocaml_record!`]), an
        `OCaml<YourRecordMarker>` (where `YourRecordMarker` is the Rust marker type for the OCaml
        record) can be converted to an instance of your Rust struct using the `.to_rust()` method.
    *   **From Rust to OCaml:** Similarly, with the [`ToOCaml<T>`](ToOCaml) trait implemented (via
        [`impl_conv_ocaml_record!`] or [`impl_to_ocaml_record!`]), an instance of your Rust struct
        can be converted to `OCaml<YourRecordMarker>` using `.to_ocaml(cr)` or, more commonly for
        return values from exported functions, to `BoxRoot<YourRecordMarker>` using
        `.to_boxroot(cr)`.

**Example:**

For a complete buildable example demonstrating OCaml-Rust record interoperability, please see the `docs/examples/records/` directory.

### 6.6 Variants and Enums

OCaml variants, akin to Rust enums, define a type that can be one of
several distinct forms (constructors), each optionally carrying data.
The `ocaml-interop` crate provides macros to simplify conversions
between Rust enums and OCaml variants.

**Key Concepts:**

*   **Rust Enum Definition:** Define a Rust enum where each variant mirrors an
    OCaml constructor.
    *   OCaml constructors without arguments (e.g., `| Nothing`) map to Rust
        enum variants without fields (e.g., `Nothing`).
    *   OCaml constructors with a single argument (e.g., `| IntVal of int`)
        map to Rust enum variants with a single field (e.g., `IntVal(i64)`).
        Note that OCaml `int` is typically represented as `i64` in Rust fields.
    *   OCaml constructors with multiple arguments, often represented as a
        tuple in OCaml (e.g., `| PairVal of string * int`), map to Rust enum
        variants with corresponding fields, often a tuple (e.g.,
        `PairVal(String, i64)`).

*   **Order is Crucial:** The most critical aspect when mapping OCaml variants
    is that **the order of variants declared within the conversion macros
    must exactly match the order of constructors in the OCaml variant type
    definition.** OCaml assigns tags to variant constructors based on this
    order (separately for constructors with and without arguments), and the
    macros rely on this positional correspondence.

*   **[`impl_conv_ocaml_variant!`] Macro:** This is the primary macro for
    enabling bidirectional conversion. It generates [`ToOCaml`] (Rust to OCaml)
    and [`FromOCaml`] (OCaml to Rust) trait implementations.
    *   If only one-way conversion is needed, [`impl_to_ocaml_variant!`] or
        [`impl_from_ocaml_variant!`] can be used directly.

*   **Macro Syntax:**
    The macro requires the Rust enum name, an optional OCaml marker type name
    (if different), and then a list of the Rust enum's variants in the
    correct order.

    ```rust
    # use ocaml_interop::*;
    // OCaml definition:
    // type status =
    //   | Ok                (* First constructor, no arguments *)
    //   | Error of string   (* Second constructor, one argument *)
    //   | Retrying of int   (* Third constructor, one argument *)

    #[derive(Debug, PartialEq)]
    pub enum Status {
        Ok,
        Error(String),
        Retrying(i64),
    }

    // Rust marker type for the OCaml `status`
    pub enum OCamlStatus {}

    impl_conv_ocaml_variant!(Status => OCamlStatus {
        // Variants must be listed in the same order as in OCaml.
        // For `impl_to_ocaml_variant` (Rust -> OCaml):
        //   RustEnum::VariantName(payload_name: OCamlRepresentationForPayloadType, ...)
        // For `impl_from_ocaml_variant` (OCaml -> Rust):
        //   RustEnum::VariantName(payload_name: OCamlRepresentationForPayloadType, ...)

        Status::Ok, // No payload
        Status::Error(message: String), // Payload `message` is String in OCaml
        Status::Retrying(count: OCamlInt), // Payload `count` is OCaml `int` (Rust i64)
                                           // `OCamlInt` is used for the interop layer.
    });
    ```

    *   **Variant Listing Details:**
        *   List the fully qualified Rust enum variants (e.g., `MyEnum::VariantName`).
        *   For variants with payloads:
            `MyEnum::VariantName(name1: OCamlType1, name2: OCamlType2, ...)`
            The `name`s are identifiers for the payload values. The `OCamlType`s
            specify the OCaml representation used during conversion (e.g.,
            `OCamlInt` for an OCaml `int` that corresponds to a Rust `i64`
            field, `String` for OCaml `string`).
        *   For variants without payloads: Just `MyEnum::VariantName`.
    *   **Marker Type:** Use `RustEnumName => OCamlMarkerTypeName` if the Rust
        enum and the OCaml marker type (used as `OCaml<OCamlMarkerTypeName>`)
        have different names. If they are the same, `MyEnum { ... }` suffices.

*   **Conversion Methods:**
    *   **From OCaml to Rust:** An `OCaml<YourVariantMarker>` is converted to
        your Rust enum using `.to_rust()`.
    *   **From Rust to OCaml:** An instance of your Rust enum is converted to
        `OCaml<YourVariantMarker>` using `.to_ocaml(cr)` or to
        `BoxRoot<YourVariantMarker>` using `.to_boxroot(cr)`.

*   **OCaml Variant Tags:**
    *   OCaml constructors without arguments are represented as
        immediate integers. Their tag is effectively their index among
        other argument-less constructors (0, 1, 2...).
    *   OCaml constructors with arguments are represented as "blocks" (a
        pointer to a memory region). Their tag is their index among other
        argument-bearing constructors (0, 1, 2...).
    *   The `ocaml-interop` macros correctly derive and use these tags based
        on the strict ordering of variants provided in the macro invocation.

**Example:**

For a complete buildable example demonstrating OCaml-Rust variant/enum interoperability, please see the `docs/examples/variants/` directory.

### 6.7 Polymorphic Variants

Interaction with OCaml polymorphic variants is supported via specific macros
that leverage name-based matching.

**Key Concepts:**

1.  **Name-Based Matching:**
    *   Unlike regular variants, polymorphic variants are matched by their
        **name** (e.g., Rust `MyVariant` for OCaml `` `MyVariant ``). Names must
        match exactly, including casing (e.g., `Set_speed` in Rust for
        `` `Set_speed `` in OCaml).
    *   The order of constructors in OCaml or Rust definitions is not
        significant for the mapping, only the names.

2.  **Rust Enum Equivalence:**
    *   A corresponding Rust enum is typically defined to represent the
        OCaml polymorphic variant.

3.  **Conversion Macros:**
    *   [`impl_to_ocaml_polymorphic_variant!`]: Implements [`ToOCaml`] for
        converting a Rust enum to an OCaml polymorphic variant.
        *   Requires an exhaustive match for all Rust enum variants.
        *   Specify Rust field names and OCaml types for payloads.
    *   [`impl_from_ocaml_polymorphic_variant!`]: Implements [`FromOCaml`] for
        converting an OCaml polymorphic variant to a Rust enum.
        *   Match OCaml constructor names; specify payload types and
            conversion to Rust (e.g., `field.to_rust()`).
    *   Note: Unlike regular variants, there is no single `impl_conv_` macro;
        use the `to_` and `from_` macros separately for bidirectional conversion.

**Example:**

For a complete buildable example demonstrating OCaml-Rust polymorphic variant interoperability, please see the `docs/examples/polymorphic_variants/` directory.

### 6.8 Bigarrays (Placeholder)

[Content to be added later. This section will explain how to work with OCaml Bigarrays for
efficient numerical data exchange.]

### 6.9 Threading Considerations (Placeholder)

[Content to be added later. This section will discuss best practices for using `ocaml-interop`
in a multi-threaded OCaml 5 environment with domains.]

### 6.10 The `noalloc` Attribute In-Depth (Placeholder)

[Content to be added later. This section will cover advanced details, restrictions, and best practices for the `noalloc` attribute.]
