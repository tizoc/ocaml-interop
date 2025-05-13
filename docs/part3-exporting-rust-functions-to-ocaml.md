## Part 3: Exporting Rust Functions to OCaml

The [`#[ocaml_interop::export]`](export) macro is the designated mechanism for rendering Rust code callable
from OCaml.

### 3.1 The `#[ocaml_interop::export]` Macro

```rust
use ocaml_interop::{OCaml, OCamlRuntime, OCamlBytes, OCamlInt, ToOCaml};

#[ocaml_interop::export]
fn process_bytes(cr: &mut OCamlRuntime, data: OCaml<OCamlBytes>) -> OCaml<OCamlInt> {
    let byte_slice: &[u8] = &data.to_rust::<Vec<u8>>();
    let length = byte_slice.len() as i64;
    length.to_ocaml(cr)
}
```
-   The macro generates an `extern "C"` function possessing the same identifier as the Rust
    function.
-   It handles the necessary FFI boilerplate for argument and return value passing between OCaml
    and Rust, ensuring type-safe exchange. It does not perform automatic data conversion;
    instead, it provides safe Rust abstractions (like [`OCaml<T>`](OCaml) and [`BoxRoot<T>`](BoxRoot)) over OCaml values.
    Explicit conversions (e.g., using [`.to_rust()`](OCaml::to_rust)) are performed by user code within the function.
-   The initial argument *must* be [`cr: &mut OCamlRuntime`](OCamlRuntime).

### 3.2 Argument Types and Rooting Considerations

When OCaml invokes an exported Rust function:
-   **[`OCaml<'gc, T>`](OCaml):** Used for OCaml values passed as arguments. These are *not* automatically
    rooted by the macro within the Rust function body. Their lifetime `'gc` is bound to the scope
    of `cr` for that specific invocation. If persistence beyond this scope or re-passing to OCaml
    is required, explicit rooting (e.g., by creating a `BoxRoot<T>`) is mandatory.
-   **[`BoxRoot<T>`](BoxRoot):** If an argument is declared as `BoxRoot<T>`, the [`#[ocaml_interop::export]`](export)
    macro automatically roots the incoming OCaml value (which is assumed to correspond to `T`)
    and provides it as a `BoxRoot<T>` *before* your function body is executed. This provides
    you with a `BoxRoot<T>` that is guaranteed to be valid throughout the execution of your Rust
    function, even if your function makes further calls into the OCaml runtime (which typically
    require [`&mut OCamlRuntime`](OCamlRuntime) and could invalidate unrooted `OCaml<'gc, T>` values). This is
    particularly useful if the function needs to hold onto the OCaml value across such calls or
    for complex operations.
    ```rust
    # use ocaml_interop::*;
    #[ocaml_interop::export]
    fn takes_rooted_string(cr: &mut OCamlRuntime, name: BoxRoot<String>) -> OCaml<OCamlInt> {
        // 'name' is a BoxRoot<String>, automatically rooted by the macro before this function body starts.
        // It remains valid even if we make OCaml calls here.
        // For example, if we called an OCaml logging function (which itself requires &mut OCamlRuntime):
        // let message_to_log = "About to process string".to_boxroot(cr); // Value must be rooted before the call
        // ocaml_api::log_something(cr, &message_to_log); // Assuming ocaml_api::log_something exists
        // After the call to ocaml_api::log_something, 'name' (the BoxRoot argument) would still be valid for use.

        let len = name.to_rust::<String>(cr).len() as i64;
        len.to_ocaml(cr)
    }
    ```
-   **Direct Primitive Type Arguments:** Certain Rust primitive types can be passed directly as arguments. For a detailed list and explanation of these mappings, see Section 3.7 "Direct Primitive Type Mapping".

### 3.3 Return Types

-   **[`OCaml<T>`](OCaml):** Typically, functions return [`OCaml<T>`](OCaml). The macro handles the conversion of
    this to a [`RawOCaml`] value suitable for OCaml. The returned value must be valid within the
    current `cr` scope.
-   **Direct Primitive Type Returns:** Certain Rust primitive types can be returned directly. For a detailed
    list and explanation of these mappings, see Section 3.7 "Direct Primitive Type Mapping". This
    includes types like `f64`, `i64`, `i32`, `bool`, `isize`, and `()`.

### 3.4 Panic Handling Mechanisms

-   **Default Behavior:** If an exported Rust function panics, [`#[ocaml_interop::export]`](export)
    intercepts the panic.
    1.  It attempts to raise a specific OCaml exception: `RustPanic of string`. This exception
        must be defined and registered within the OCaml codebase:
        ```ocaml
        exception RustPanic of string
        let () = Callback.register_exception "rust_panic_exn" (RustPanic "")
        ```
    2.  If `RustPanic` (registered under the name `"rust_panic_exn"`) is not found, the macro
        defaults to raising OCaml's standard `Failure` exception, incorporating the panic
        message.
-   This mechanism prevents Rust panics from unwinding across the FFI boundary, which would
    otherwise lead to undefined behavior.
-   **`no_panic_catch` Attribute:**
    ```rust
    # use ocaml_interop::*;
    #[ocaml_interop::export(no_panic_catch)]
    fn my_critical_function(cr: &mut OCamlRuntime, arg: OCaml<String>, /* ... */) { /* ... */ }
    ```
    This attribute should be used if it is certain that the function will not panic, or for
    highly specialized error handling scenarios. It disables the automatic panic interception.
    **Employ with extreme caution.**

### 3.5 Bytecode Function Generation

For OCaml projects targeting bytecode compilation, a compatible wrapper function can be generated.
-   **`bytecode = "my_ocaml_bytecode_function_name"` Attribute:**
    ```rust
    # use ocaml_interop::*;
    #[ocaml_interop::export(bytecode = "rust_twice_bytecode")]
    fn rust_twice(cr: &mut OCamlRuntime, num: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
        // ...
        # num
    }
    ```
-   **OCaml Declaration:**
    ```ocaml
    external rust_twice : int -> int = "rust_twice_bytecode" "rust_twice"
    (*       ^ OCaml type          ^ bytecode stub name    ^ native C stub name *)
    ```
    This directs OCaml to use `rust_twice_bytecode` for bytecode execution and `rust_twice`
    (the default native function name) for native execution.

### 3.6 The `noalloc` Attribute

For performance-sensitive FFI calls where OCaml garbage collector (GC) allocations must be avoided,
the `noalloc` attribute can be used. This aligns with OCaml's `[@@noalloc]` attribute on
`external` function declarations.

-   **Syntax:**
    ```rust
    # use ocaml_interop::*;
    #[ocaml_interop::export(noalloc)]
    fn my_no_alloc_function(cr: &OCamlRuntime, arg: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
        // Function body must not allocate OCaml values or trigger GC
        // ...
        arg // Example: returning an input directly
    }
    ```

-   **Key Effects and Requirements:**
    *   **Immutable Runtime:** The Rust function *must* take an immutable reference to the OCaml
        runtime: [`&OCamlRuntime`](OCamlRuntime). The macro will produce a compile-time error if [`&mut OCamlRuntime`](OCamlRuntime)
        is used with `noalloc`, or if [`&OCamlRuntime`](OCamlRuntime) is used without `noalloc`.
    *   **Internal Runtime Handle:** The macro uses `::ocaml_interop::internal::recover_runtime_handle()`
        to obtain an immutable runtime handle, rather than the mutable version.
    *   **Implies `no_panic_catch`:** Functions marked with `noalloc` automatically have panic
        catching disabled, similar to specifying `no_panic_catch`. Any panic in a `noalloc`
        function will unwind across the FFI boundary, leading to undefined behavior.
        **This requires extreme care to ensure the function cannot panic.**

-   **User Responsibilities:**
    *   **No OCaml Allocations:** The Rust function body must strictly avoid any operations that
        could lead to OCaml memory allocation or interact with the OCaml GC in a way that
        requires mutable runtime access. This includes not calling OCaml functions that might
        allocate, and not creating new OCaml values that require allocation (e.g., most `OCaml<T>`
        or `BoxRoot<T>` creations from Rust types).
    *   **OCaml `[@@noalloc]` Annotation:** The corresponding OCaml `external` declaration for the
        Rust function **must** be annotated with `[@@noalloc]`. For example:
        ```ocaml
        external my_no_alloc_function : int -> int = "my_no_alloc_function" [@@noalloc]
        ```
    *   The user is responsible for upholding the contract of `[@@noalloc]` on both the Rust and
        OCaml sides.

Use `noalloc` only for very specific, performance-critical paths. When an OCaml `external`
is marked with `[@@noalloc]`, the OCaml native-code compiler omits its usual bookkeeping code
(e.g., `caml_c_call` wrapper) around the C call. This makes the FFI call as cheap as a direct
OCaml function call, significantly reducing overhead for small, frequently called functions.
This is beneficial when the Rust function is guaranteed not to allocate OCaml memory, raise
exceptions, or release the OCaml domain lock, and the standard FFI overhead is prohibitive.
Ensure the function's logic strictly adheres to these no-allocation constraints.

### 3.7 Direct Primitive Type Mapping

Certain Rust primitive types can be directly mapped to unboxed or untagged OCaml types for both
function arguments and return values. This approach can offer performance benefits by avoiding the
overhead of boxing these values. The table below summarizes these direct mappings:

| Rust Type | OCaml Type        | OCaml `external` Attribute(s) Needed |
| :-------- | :---------------- | :----------------------------------- |
| `f64`     | `float`           | `[@@unboxed]` (or on arg/ret type)   |
| `i64`     | `int64`           | `[@@unboxed]` (or on arg/ret type)   |
| `i32`     | `int32`           | `[@@unboxed]` (or on arg/ret type)   |
| `bool`    | `bool`            | `[@untagged]` (or on arg/ret type)    |
| `isize`   | `int`             | `[@untagged]` (or on arg/ret type)    |
| `()`      | `unit`            | (Usually implicit for return; can be used for arguments) |

**Note:** When using `[@@unboxed]` on the OCaml `external` function declaration, it applies to all
eligible arguments and the return type. Alternatively, attributes like `(float [@unboxed])` or
`(int [@untagged])` can be applied to specific arguments in the OCaml `external` signature.

For example, a Rust function using these direct primitives:

```rust
# use ocaml_interop::*;
#[ocaml_interop::export]
fn process_primitive_values(cr: &mut OCamlRuntime, count: isize, active: bool, value: f64) -> i32 {
    if active {
        println!("Processing count: {}, value: {}", count, value);
        (count as i32) + (value as i32)
    } else {
        0
    }
}
```

And its corresponding OCaml `external` declaration:

```ocaml
external process_primitive_values :
  (int [@untagged]) ->
  (bool [@untagged]) ->
  (float [@unboxed]) ->
  (int32 [@unboxed]) =
  "" "process_primitive_values"
```
