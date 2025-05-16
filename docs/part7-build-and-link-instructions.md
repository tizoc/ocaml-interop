# Part 7: Build and Link Instructions

This section outlines how to prepare and build OCaml and Rust projects using `ocaml-interop`.

## 7.1 OCaml Programs Calling Rust Code

This section details how to structure and build an OCaml executable that calls functions from a Rust library, leveraging Dune to manage the Rust compilation and linking. This approach is based on the patterns observed in project examples like `testing/ocaml-caller` and `docs/examples/*`.

### 7.1.1 Project Structure Overview

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

### 7.1.2 Rust Library Configuration (`rust_lib/Cargo.toml`)

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

### 7.1.3 Dune Configuration for Building Rust Code (`rust_lib/dune`)

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

### 7.1.4 Main OCaml Executable Configuration (`ocaml_project/dune`)

The Dune file for the main OCaml executable needs to:
- Reference the OCaml library defined in `rust_lib/dune` (e.g., `my_rust_lib_crate`).
- Include `threads`.

```dune
; ocaml_project/dune
(executable
 (name my_ocaml_app)
 (modules my_ocaml_app)
 (libraries
  my_rust_lib_crate ; Refers to the (name ...) in rust_lib/dune
  threads
  (* other ocaml dependencies *)
 )
)
```

### 7.1.5 Build Process

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

## 7.2 Rust Programs Calling OCaml Code

This section explains how to build Rust executables or libraries that call functions written in OCaml. It involves compiling the OCaml code into a static library, using a `build.rs` script in the Rust project to manage the OCaml compilation and linking, and correctly handling the OCaml runtime and FFI declarations.

We'll use the `ocaml-interop-dune-builder` crate to help with the Dune compilation step.

### 7.2.1 Example Project Structure

Consider a Rust project that wants to use an OCaml library:

```text
my_rust_app/
├── Cargo.toml
├── build.rs
├── src/
│   └── main.rs
└── ocaml_math_lib/      <-- OCaml library source code
    ├── dune-project     <-- Defines the Dune project context (can be minimal)
    ├── dune             <-- Dune file for building the OCaml library
    └── math_ops.ml      <-- OCaml implementation
```

### 7.2.2 OCaml Library Preparation (`ocaml_math_lib/`)

The OCaml code needs to be compiled into a static library (`.a` file). Functions intended to be called from Rust should be registered with OCaml's `Callback` mechanism.

#### 7.2.2.1 `ocaml_math_lib/dune-project`
A minimal `dune-project` file:
```dune
(lang dune 3.7)
```

#### 7.2.2.2 `ocaml_math_lib/dune`
This file defines how to build the OCaml object files.
```dune
; ocaml_math_lib/dune

(executables
 (names math_ops_lib) ; Base name for the target
 (modules math_ops)   ; OCaml modules to compile (math_ops.ml)
 (modes object)       ; Compile to object files, not a linked executable
 (libraries threads)  ; `threads` library is required
)
```
This Dune configuration will compile `math_ops.ml` into object files (like `math_ops.cmx`, `math_ops.o`). The `(modes object)` stanza is key here. The output object files will typically be in `ocaml_math_lib/_build/default/` (or a similar path depending on the profile and if `ocaml_math_lib` is the root of the dune project).

To make OCaml functions callable from Rust, you would typically register them in your OCaml code, for example, in `math_ops.ml`:
```ocaml
(* ocaml_math_lib/math_ops.ml *)
let add_ints (a : int) (b : int) : int = a + b

let () =
  Callback.register "add_ints_ocaml" add_ints
```

### 7.2.3 Rust `build.rs` Configuration

The `build.rs` script in your Rust project (`my_rust_app/build.rs`) will use `ocaml-interop-dune-builder` to compile the OCaml library and then instruct Cargo how to link against it.

First, add `ocaml-interop-dune-builder` to your `[build-dependencies]` in `my_rust_app/Cargo.toml`:
```toml
[package]
name = "my_rust_app"
version = "0.1.0"
edition = "2021"

[dependencies]
ocaml-interop = "*" # API currently unstable, use a specific version

[build-dependencies]
ocaml-interop-dune-builder = "*"
cc = "1.0" # For compiling the .o files into a static library
```

Now, the `my_rust_app/build.rs` script:
```rust,ignore
// my_rust_app/build.rs
use ocaml_interop_dune_builder::DuneBuilder;
use std::env;

fn main() {
    let ocaml_lib_source_dir = "ocaml_math_lib";

    // --- Instruct cargo to rerun when OCaml files change ---
    println!("cargo:rerun-if-changed={}/dune", ocaml_lib_source_dir);
    println!("cargo:rerun-if-changed={}/math_ops.ml", ocaml_lib_source_dir);

    // --- Build the OCaml library using DuneBuilder ---
    let builder = DuneBuilder::new(ocaml_lib_source_dir);
        // .with_profile("release") // For release builds
        // .with_dune_invocation(DuneInvocation::System) // If opam exec is not desired

    // The target for Dune to build. For `(executables (names math_ops_exe) (modes object))`,
    // a common target that ensures all necessary .o files are built is `<name>.exe.o` or just `<name>.exe`.
    // DuneBuilder will then collect the .o files from the output directory.
    let dune_target = "math_ops.exe.o";
    // Or, depending on your Dune setup and what you want to build, it could be just `ocaml_lib_name`
    // if that target is defined to produce the necessary objects.

    // This command executes `dune build <dune_target>` within the ocaml_math_lib directory.
    // `build()` returns a list of .o files found in the build output directory for ocaml_lib_source_dir.
    let o_files = builder.build(&dune_target);

    if o_files.is_empty() {
        eprintln!("Warning: DuneBuilder found no .o files for target {}.", dune_target);
        eprintln!("Check that the dune target produces .o files in the expected directory:");
        eprintln!("  <ocaml_lib_source_dir>/_build/<profile>/");
        // Potentially panic here if .o files are essential and not found.
    }

    // --- Compile the collected .o files into a static library ---
    // The `cc` crate will compile these OCaml .o files (and any C stubs if they were also collected)
    // into a static library (e.g., libocaml_math_compiled_archive.a) and tell Cargo how to link it.
    let mut cc_build = cc::Build::new();
    for o_file in &o_files {
        cc_build.object(o_file);
    }

    // Define the name of the static library that `cc::Build` will create.
    // This is the name Rust will link against.
    let archive_name = "ocaml_math_compiled_archive";
    cc_build.compile(archive_name);

    // `cc::Build::compile` automatically prints the necessary cargo:rustc-link-search
    // and cargo:rustc-link-lib directives for the compiled archive.
    // So, we do NOT need the following manual lines anymore:
    // println!("cargo:rustc-link-search=native={}", ocaml_lib_output_dir.display());
    // println!("cargo:rustc-link-lib=static={}", ocaml_lib_name);

    // Note on OCaml runtime linking:
    // The `ocaml-sys` crate (a dependency of `ocaml-interop`) automatically handles
    // linking the OCaml runtime (libasmrun).
    // You do not need to specify them manually here.
}
```

### 7.2.4 Building and Running

With the above setup:
1.  `cargo build` will:
    a.  Execute `my_rust_app/build.rs`.
    b.  `DuneBuilder` will invoke `dune build` in `ocaml_math_lib/` to compile `math_ops_lib.exe.o`.
    c.  The `build.rs` script will output `cargo:rustc-link-search` and `cargo:rustc-link-lib` to link `math_ops_lib.a`.
    d.  `ocaml-sys` will ensure the OCaml runtime and stdlib are linked.
    e.  Cargo will compile the Rust code and link everything together.
2.  Run the executable: `target/debug/my_rust_app`.
