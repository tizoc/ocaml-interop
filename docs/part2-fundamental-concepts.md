## Part 2: Fundamental Concepts

A thorough understanding of these core types is essential for the effective utilization of [`ocaml-interop`].

### 2.1 Representing OCaml Values within Rust

-   **[`OCaml<'gc, T>`](OCaml):** This type serves as the primary wrapper for an OCaml value within Rust code.
    -   `'gc`: Represents a lifetime parameter associated with an active OCaml runtime scope. This
        signifies that the OCaml Garbage Collector (GC) may relocate or deallocate the value if it
        is not "rooted."
    -   `T`: Denotes the Rust type corresponding to the OCaml value (e.g., [`OCamlInt`], `String`,
        [`OCamlList<OCamlFloat>`](OCamlList)).
    -   Instances of this type are generally ephemeral and should be regarded as potentially invalid
        subsequent to any invocation into the OCaml runtime, unless explicitly rooted.

-   **[`BoxRoot<T>`](BoxRoot):** A smart pointer that "roots" an OCaml value, thereby ensuring that the OCaml
    GC does not deallocate or move it while the [`BoxRoot<T>`](BoxRoot) instance persists.
    -   It adheres to the RAII (Resource Acquisition Is Initialization) principle: the OCaml value
        is automatically unrooted when the [`BoxRoot<T>`](BoxRoot) instance is dropped.
    -   This mechanism is crucial for safely retaining OCaml values across multiple operations or
        Rust scopes.
    -   Instances are created by:
        - Calling [`.to_boxroot(cr)`](ToOCaml::to_boxroot) on a Rust value (e.g., `rust_value.to_boxroot(cr)`), which
          converts it to an OCaml value and roots it.
        - Calling [`.root()`](OCaml::root) on an existing [`OCaml<T>`](OCaml) value (e.g., `ocaml_val.root()`).
        - Using [`BoxRoot::new(ocaml_val)`](BoxRoot::new) with an existing [`OCaml<T>`](OCaml) value. Note that
          `BoxRoot::new()` will panic if the underlying boxroot allocation fails.
    -   [`BoxRoot<T>`](BoxRoot) is `!Send` and `!Sync` due to its direct interaction with OCaml's
        domain-specific GC state and memory management, meaning it cannot be safely transferred
        across threads or shared concurrently between threads.

-   **[`RawOCaml`]:** An unsafe, raw pointer-sized type representing an OCaml value. Direct
    interaction with this type is infrequent, as [`OCaml<T>`](OCaml) and [`BoxRoot<T>`](BoxRoot) provide safe
    abstractions.

### 2.2 Converting Data Between Rust and OCaml

The traits [`ToOCaml<T>`](ToOCaml) and [`FromOCaml<T>`](FromOCaml) are provided to facilitate data conversions.

-   **[`ToOCaml<OCamlType>`](ToOCaml):**
    -   Implemented by Rust types that are convertible to OCaml values of `OCamlType`.
    -   Provides the method [`.to_ocaml(cr: &mut OCamlRuntime)`](ToOCaml::to_ocaml) to create an
        OCaml value.
    -   Provides the method [`.to_boxroot(cr: &mut OCamlRuntime)`](ToOCaml::to_boxroot) to create
        a rooted OCaml value.
    -   Example: `let ocaml_string: OCaml<String> = "hello".to_ocaml(cr);`;

-   **[`FromOCaml<OCamlType>`](FromOCaml):**
    -   Implemented by Rust types that can be instantiated from OCaml values of `OCamlType`.
    -   [`OCaml<T>`](OCaml) provides [`.to_rust::<RustType>()`](OCaml::to_rust).
    -   [`BoxRoot<T>`](BoxRoot) provides `.to_rust::<RustType>(cr: &mut OCamlRuntime)`.
    -   Example: `let rust_int: i64 = ocaml_int_value.to_rust();`

**Common Type Mappings:**
-   Rust `i64` corresponds to OCaml `int` (represented as [`OCamlInt`]).
-   Rust `f64` corresponds to OCaml `float` (represented as [`OCamlFloat`]).
-   Rust `String`/`&str` corresponds to OCaml `string`.
-   Rust `Vec<T>` corresponds to OCaml `list` or `array` (e.g., [`OCamlList<T>`](OCamlList),
    [`OCamlUniformArray<T>`](OCamlUniformArray)).
-   Rust `Option<T>` corresponds to OCaml `option`.
-   Rust `Result<T, E>` corresponds to OCaml `result` (often with [`OCaml<String>`](OCaml) for error types).
