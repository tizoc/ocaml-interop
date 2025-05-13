## Part 5: Managing the OCaml Runtime (for Rust-driven programs)

If a program originates in Rust and requires interaction with OCaml code, the OCaml runtime must
be explicitly managed by the Rust code.
**Conversely, if OCaml invokes Rust code as a library, the OCaml runtime will have already been
initialized; [`OCamlRuntime::init()`](OCamlRuntime::init) must not be called in such scenarios.**

### 5.1 Runtime Initialization

-   **[`OCamlRuntime::init`]`() -> Result<`[`OCamlRuntimeStartupGuard`]`, String>`:**
    -   Initializes the OCaml C runtime and the `boxroot` system for GC interoperability.
    -   This function should be called once at the startup of the Rust program.
-   **[`OCamlRuntimeStartupGuard`]:**
    -   An RAII guard instance returned by [`OCamlRuntime::init()`](OCamlRuntime::init).
    -   Upon being dropped, this guard automatically executes
        `boxroot_teardown` and `caml_shutdown` to ensure proper cleanup of the OCaml runtime.
    -   It is `!Send` and `!Sync`.

```rust,no_run
use ocaml_interop::{OCamlRuntime, OCamlRuntimeStartupGuard};

fn main() -> Result<(), String> {
    let _guard: OCamlRuntimeStartupGuard = OCamlRuntime::init()?;
    // The OCaml runtime is now active.
    // ... OCaml operations are performed here ...
    Ok(())
} // _guard is dropped at this point, thereby shutting down the OCaml runtime.
```

### 5.2 Acquiring the Domain Lock

Most OCaml operations mandate that the current thread holds the OCaml domain lock.
-   **[`OCamlRuntime::with_domain_lock`]`(|cr: &mut OCamlRuntime| { /* ... */ })`:**
    -   This is the canonical method for obtaining the [`&mut OCamlRuntime`](OCamlRuntime) handle (`cr`).
    -   It ensures that the current thread is registered as an OCaml domain and acquires the OCaml
        runtime lock. The lock is released upon completion of the closure.
    -   All OCaml interactions should typically be performed within this closure.

```rust,no_run
# use ocaml_interop::{OCamlRuntime, OCamlRuntimeStartupGuard, ToOCaml, OCaml};
# fn main() -> Result<(), String> {
# let _guard = OCamlRuntime::init()?;
OCamlRuntime::with_domain_lock(|cr| {
    // 'cr' is the &mut OCamlRuntime.
    // All OCaml interactions are performed here.
    let _ocaml_string: OCaml<String> = "Hello, OCaml!".to_ocaml(cr);
    // ... invoke functions from the ocaml! macro, etc. ...
});
# Ok(())
# }
```
