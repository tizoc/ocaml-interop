# `ocaml-interop-derive`

This crate provides the `#[ocaml_interop::export]` attribute macro, a key component of the `ocaml-interop` library. It is responsible for generating the necessary boilerplate to expose Rust functions to the OCaml runtime.

**Note:** `ocaml-interop-derive` is an internal dependency of the main `ocaml-interop` crate. Users should typically depend on `ocaml-interop` and use the macro through it (e.g., `#[ocaml_interop::export]`).

## `#[ocaml_interop::export]` Macro

The `#[ocaml_interop::export]` attribute macro is used to expose Rust functions to the OCaml runtime. It handles the necessary transformations to make a Rust function callable as if it were an OCaml C FFI function.

### Features:

*   **Automatic Runtime Handle**: The first argument of your Rust function must be the OCaml runtime handle (e.g., `cr: &mut ocaml_interop::OCamlRuntime`). The macro verifies this and uses it internally for operations like panic handling and type conversions.
*   **Argument Marshalling**:
    *   Arguments of type `f64` are passed directly.
    *   OCaml values passed as arguments (e.g., OCaml `string`, `int`, custom types) are made available in Rust as `ocaml_interop::OCamlRef<T>`. This provides a safe, temporary reference to the OCaml data. You can then use methods like `.to_rust(cr)` to convert them to Rust types or `.root(cr)` to create a `BoxRoot<T>` if the value needs to outlive the current function call.
*   **Return Value Marshalling**:
    *   If your Rust function returns `f64`, it's returned directly to OCaml.
    *   If your Rust function returns `ocaml_interop::OCaml<T>`, its underlying raw OCaml value is returned to OCaml.
    *   If your Rust function returns `()` (unit), it's implicitly treated as returning `ocaml_interop::OCaml::unit()`, which corresponds to OCaml's `unit`.
*   **Panic Handling**:
    *   By default, the macro wraps the body of the exported Rust function in `std::panic::catch_unwind`.
    *   If a panic occurs within the Rust function, it is caught.
    *   The panic message is then used to raise an OCaml exception.
    *   It first attempts to raise a custom OCaml exception (e.g., `RustPanic of string`) if one has been registered from the OCaml side with the name `"rust_panic_exn"`. (See OCaml `Callback.register_exception`).
    *   If the custom exception `"rust_panic_exn"` is not found, it falls back to raising a standard OCaml `Failure` exception (via `caml_failwith`) with the panic message.
    *   This panic handling can be disabled by adding the `no_panic_catch` attribute: `#[ocaml_interop::export(no_panic_catch)]`.
*   **FFI Signature Generation**: The macro generates the `#[no_mangle]` and `pub extern "C"` function with the correct FFI signature, making it callable from OCaml.
*   **Bytecode Wrapper Generation**:
    *   Optionally, the macro can also generate a second wrapper function compatible with OCaml bytecode calling conventions.
    *   This is enabled by providing a `bytecode` attribute with the desired name for the bytecode stub: `#[ocaml_interop::export(bytecode = "my_function_bytecode_stub")]`.
    *   The generated C FFI function will then call this bytecode stub. This is useful when the OCaml code might be compiled to bytecode.

### Usage Examples

Below are examples of how to use the `#[ocaml_interop::export]` macro.

```rust
// In your lib.rs or a module, assuming ocaml_interop is a dependency.
use ocaml_interop::{OCamlRuntime, OCamlRef, OCaml, OCamlInt, ToOCaml, FromOCaml};

// Example 1: A simple function with no extra arguments and returning unit (implicitly OCaml<()>)
#[ocaml_interop::export]
fn rust_hello_world(cr: &mut OCamlRuntime) {
    println!("Hello from Rust!");
    // Implicitly returns OCaml::unit()
}

// Example 2: Function with an OCamlInt argument and returning an OCamlInt
#[ocaml_interop::export]
fn rust_double_int(cr: &mut OCamlRuntime, num: OCamlRef<OCamlInt>) -> OCaml<OCamlInt> {
    let rust_num: i64 = num.to_rust(cr);
    let result = rust_num * 2;
    OCaml::of_i64_unchecked(result) // Creates an OCaml<OCamlInt>
}

// Example 3: Function with an f64 argument and returning an f64
#[ocaml_interop::export]
fn rust_add_pi(cr: &mut OCamlRuntime, val: f64) -> f64 {
    val + std::f64::consts::PI
}

// Example 4: Function with multiple arguments (OCamlRef and f64)
#[ocaml_interop::export]
fn rust_process_data(
    cr: &mut OCamlRuntime,
    label: OCamlRef<String>, // OCaml string
    value: OCamlRef<OCamlInt>, // OCaml int
    factor: f64,
) -> OCaml<String> { // Returns OCaml string
    let r_label: String = label.to_rust(cr); // Convert OCamlRef<String> to Rust String
    let r_value: i64 = value.to_rust(cr);    // Convert OCamlRef<OCamlInt> to Rust i64
    let processed_value = (r_value as f64 * factor) as i64;
    let result_string = format!("{}: {}", r_label, processed_value);
    result_string.to_ocaml(cr) // Convert Rust String back to OCaml<String>
}

// Example 5: Function that might panic
#[ocaml_interop::export]
fn rust_might_panic(cr: &mut OCamlRuntime, should_panic: OCamlRef<bool>) {
    if should_panic.to_rust(cr) {
        panic!("This is a deliberate panic from Rust!");
    }
    println!("Did not panic.");
    // Implicitly returns OCaml::unit()
}

// How these functions would be declared on the OCaml side:
/*
external rust_hello_world: unit -> unit = "rust_hello_world"
external rust_double_int: int -> int = "rust_double_int"
external rust_add_pi: float -> float = "rust_add_pi"
external rust_process_data: string -> int -> float -> string = "rust_process_data"
external rust_might_panic: bool -> unit = "rust_might_panic"
  (* If rust_might_panic panics, OCaml will see:
     - `exception RustPanic of string` with payload "This is a deliberate panic from Rust!" (if registered as "rust_panic_exn")
     - `Failure "This is a deliberate panic from Rust!"` (otherwise) *)
*/
```

This `README.md` provides a basic overview. For more detailed information on `ocaml-interop` concepts like `OCamlRuntime`, `OCamlRef`, `OCaml<T>`, `BoxRoot`, type conversions (`ToOCaml`, `FromOCaml`), and OCaml exception registration, please refer to the main `ocaml-interop` crate documentation.
