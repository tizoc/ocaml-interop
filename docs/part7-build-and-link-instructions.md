# Part 7: Build and Link Instructions

This section outlines how to prepare and build OCaml and Rust projects using `ocaml-interop`.

## Section 1: OCaml Programs Calling Rust Code

This section details how to structure and build an OCaml executable that calls functions from a Rust library, leveraging Dune to manage the Rust compilation and linking. This approach is based on the patterns observed in project examples like `testing/ocaml-caller` and `docs/examples/*`.

### 1. Project Structure Overview

A common layout involves an OCaml application and a Rust library, often as a subdirectory within the OCaml project or a related directory in a monorepo structure.

Example:

```text
ocaml_project/
├── dune-project
├── dune
├── my_ocaml_app.ml
└── rust_lib/
    ├── Cargo.toml  <-- Standard Cargo.toml for the Rust library
    ├── dune        <-- Special dune file to build the Rust library
    └── src/
        └── lib.rs
```

### 2. Rust Library Configuration (`rust_lib/Cargo.toml`)

The Rust library's `Cargo.toml` should specify `staticlib` and/or `cdylib` as crate types. The `name` of the package is crucial as it's used by Dune.

```toml
# ocaml_project/rust_lib/Cargo.toml
[package]
name = "my_rust_lib_crate" # This name will be used in dune files
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["staticlib", "cdylib"]

[dependencies]
ocaml-interop = { path = "../../.." } # Adjust path as needed
```

### 3. Dune Configuration for Building Rust Code (`rust_lib/dune`)

This Dune file, located within the Rust library's directory, instructs Dune on how to build the Rust static library using Cargo. It defines a rule to produce the static library (`.a` file) and, if needed, a shared library (`.so` or `.dylib`).

Key aspects:
- It directly uses the `Cargo.toml` file located in the `rust_lib` directory.
- It runs `cargo build`.
- It copies the compiled artifacts (`lib<name>.a`, `lib<name>.so`/`lib<name>.dylib`) to the current directory for Dune to find.
- It defines an OCaml library stanza that wraps the foreign Rust archive.

```dune
; ocaml_project/rust_lib/dune

; Rule to build the Rust library using Cargo
(rule
 (targets libmy_rust_lib_crate.a dllmy_rust_lib_crate.so) ; Or .dylib on macOS for the .so
 (deps (source_tree src) Cargo.toml) ; Depends on Rust source and the Cargo.toml file
 (action
  (no-infer
   (progn
    ;; macOS requires these flags because undefined symbols are not allowed by default
    (run sh -c "
        if [ \"$(uname -s)\" = \"Darwin\" ]; then
          export RUSTFLAGS='-C link-args=-Wl,-undefined,dynamic_lookup'
        fi
        cargo build
      ")
    (run sh -c ; Copy shared library, handling .so or .dylib
      "cp target/debug/libmy_rust_lib_crate.so ./dllmy_rust_lib_crate.so 2> /dev/null || \
       cp target/debug/libmy_rust_lib_crate.dylib ./dllmy_rust_lib_crate.so")
    (run cp target/debug/libmy_rust_lib_crate.a ./libmy_rust_lib_crate.a)
   ))))

; OCaml library stanza that makes the Rust static library available to OCaml
(library
 (name my_rust_lib_crate) ; The name OCaml executables will refer to.
                          ; Matches the Rust crate name for consistency.
 (foreign_archives my_rust_lib_crate) ; Base name of the static library (lib<name>.a)
 ; (c_library_flags -lc -lm) ; Include if your Rust code links against standard C/math libraries
                             ; or other system libraries.
)
```
**Note on `Cargo.toml`**: The `(deps ... Cargo.toml)` line in the Dune rule refers to the `Cargo.toml` file located in the same directory as this `rust_lib/dune` file (i.e., `ocaml_project/rust_lib/Cargo.toml`). Ensure this `Cargo.toml` is correctly configured for your Rust library.

### 4. Main OCaml Executable Configuration (`ocaml_project/dune`)

The Dune file for the main OCaml executable needs to:
- Reference the OCaml library defined in `rust_lib/dune` (e.g., `my_rust_lib_crate`).
- Include `threads.posix`.

```dune
; ocaml_project/dune
(executable
 (name my_ocaml_app)
 (modules my_ocaml_app)
 (libraries
  my_rust_lib_crate ; Refers to the (name ...) in rust_lib/dune
  threads.posix
  (* other ocaml dependencies *)
 )
)
```

### 5. Build Process

With this setup, Dune handles the entire build process:
1. When `dune build` is invoked for the OCaml executable:
2. Dune sees the dependency on `my_rust_lib_crate`.
3. It processes `ocaml_project/rust_lib/dune`.
4. The `(rule ...)` in `rust_lib/dune` is executed:
   a. `cargo build` compiles the Rust static library.
   b. The `.a` (and `.so`/`.dylib`) files are copied.
5. The `(library ...)` stanza in `rust_lib/dune` makes `libmy_rust_lib_crate.a` available.
6. Dune compiles the OCaml code and links it with `libmy_rust_lib_crate.a` and other specified OCaml libraries.

To build:
```bash
cd path/to/ocaml_project
dune build ./my_ocaml_app.exe
```

To run:
```bash
./_build/default/my_ocaml_app.exe
```
This integrated approach simplifies the build process, as Dune manages the compilation and linking of both OCaml and Rust components.

## Section 2: Rust Programs Calling OCaml Code

_(Placeholder for detailed instructions on building Rust programs that call OCaml code)_

This section will cover:
- Compiling OCaml code into a library suitable for linking with Rust.
- Configuring the Rust build (Cargo) to link against the OCaml library.
- Managing the OCaml runtime from Rust.
