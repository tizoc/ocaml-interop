## Part 1: Initial Usage - A Brief Overview

This section introduces basic examples to demonstrate the core functionality of [`ocaml-interop`].

### 1.1 Exporting Rust Functions to OCaml: An Introduction

Rust functions can be exposed to OCaml utilizing the [`#[ocaml_interop::export]`](export) procedural macro.

**Rust (`src/lib.rs` or designated Rust library):**
```rust
use ocaml_interop::{OCaml, OCamlInt, OCamlRuntime, ToOCaml};

#[ocaml_interop::export]
pub fn rust_add_one(cr: &mut OCamlRuntime, num: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
    let rust_num: i64 = num.to_rust();
    let result = rust_num + 1;
    result.to_ocaml(cr)
}
```

**OCaml (e.g., `main.ml`):**
```ocaml
(* Declare the external Rust function *)
external rust_add_one : int -> int = "rust_add_one"

let () =
  let five = rust_add_one 4 in
  Printf.printf "4 + 1 = %d\n" five (* Output: 4 + 1 = 5 *)
```
The [`#[ocaml_interop::export]`](export) macro manages FFI boilerplate and panic safety mechanisms. It exposes
OCaml values to Rust in a type-safe manner; subsequent conversion to Rust types must be performed
explicitly by the developer. These aspects will be detailed subsequently.

### 1.2 Invoking OCaml Functions from Rust: An Introduction

To call OCaml functions from Rust, the [`ocaml!`] macro is typically employed subsequent to the
registration of the OCaml function.

**OCaml (e.g., `my_ocaml_lib.ml`):**
```ocaml
let multiply_by_two x = x * 2

let () =
  Callback.register "multiply_by_two" multiply_by_two
```
This OCaml code must be compiled and linked with the Rust program.

**Rust (`main.rs`):**
```rust,no_run
use ocaml_interop::{
    OCaml, OCamlInt, OCamlRuntime, OCamlRuntimeStartupGuard, ToOCaml, BoxRoot
};

// Declare the OCaml function signature
mod ocaml_bindings {
    use ocaml_interop::{ocaml, OCamlInt};

    ocaml! {
        pub fn multiply_by_two(num: OCamlInt) -> OCamlInt;
    }
}

fn main() -> Result<(), String> {
    // Initialize the OCaml runtime if Rust is the primary execution context
    let _guard: OCamlRuntimeStartupGuard = OCamlRuntime::init()?;

    OCamlRuntime::with_domain_lock(|cr| {
        let rust_val: i64 = 10;

        // Pass direct Rust values - they're automatically converted
        let result_root: BoxRoot<OCamlInt> = ocaml_bindings::multiply_by_two(cr, rust_val);
        let rust_result: i64 = result_root.to_rust(cr);
        println!("10 * 2 = {}", rust_result); // Output: 10 * 2 = 20

        // Alternative: Traditional approach with explicit rooting still works
        let ocaml_val: BoxRoot<OCamlInt> = rust_val.to_boxroot(cr);
        let result_root: BoxRoot<OCamlInt> = ocaml_bindings::multiply_by_two(cr, &ocaml_val);
        let rust_result: i64 = result_root.to_rust(cr);
        println!("10 * 2 = {}", rust_result); // Output: 10 * 2 = 20
    });
    Ok(())
}
```

### 1.3 The OCaml Runtime Handle: [`OCamlRuntime`]

Interactions with the OCaml runtime require access to an [`OCamlRuntime`] instance,
conventionally named `cr`.
-   A mutable reference, [`&mut OCamlRuntime`](OCamlRuntime), is necessary for operations that may allocate OCaml
    values or trigger the OCaml Garbage Collector (GC). This is the most common requirement.
-   An immutable reference, [`&OCamlRuntime`](OCamlRuntime), can be sufficient for operations that only read from
    the OCaml heap without modifying it or causing allocations (e.g., some conversion methods on
    already existing [`OCaml<T>`](OCaml) values).
-   When a Rust function is exported using [`#[ocaml_interop::export]`](export), [`cr: &mut OCamlRuntime`](OCamlRuntime) is
    automatically supplied as the initial argument.
-   When invoking OCaml from a Rust-driven program, `cr` (typically [`&mut OCamlRuntime`](OCamlRuntime)) is
    obtained via [`OCamlRuntime::with_domain_lock(|cr| { /* ... */ })`](OCamlRuntime::with_domain_lock) following runtime
    initialization.

This handle is indispensable for managing OCaml's state, memory, and domain locks.
