# OCaml Interop Dune Builder

This crate provides a helper utility, `DuneBuilder`, for Rust build scripts (`build.rs`) that need to compile OCaml code using Dune and link the resulting object files.

It simplifies the process of:
- Locating the Dune project.
- Determining relative build paths.
- Invoking Dune to build a specific target.
- Collecting the generated object (`.o`) files.

## Usage

Add this crate as a build dependency in your `Cargo.toml`:

```toml
[build-dependencies]
ocaml-interop-dune-builder = "*"
```

Then, in your `build.rs` script:

```rust
use ocaml_interop_dune_builder::{DuneBuilder, DuneInvocation};

fn main() {
    let ocaml_source_dir = "ocaml_lib"; // Directory containing your OCaml sources relative to Cargo.toml

    // Instruct cargo to re-run the build script if OCaml files change.
    // Adjust paths and filenames as necessary.
    println!("cargo:rerun-if-changed={}/my_ocaml_lib.ml", ocaml_source_dir);
    println!("cargo:rerun-if-changed={}/dune", ocaml_source_dir); // Or specific dune files

    let objects = DuneBuilder::new(ocaml_source_dir)
        .with_profile("release") // Optional: set a build profile (default is "default")
        .with_dune_args(vec!["--display=quiet".to_string()]) // Optional: add custom dune arguments
        .with_dune_invocation(DuneInvocation::System) // Optional: invoke dune directly (default is OpamExec)
        .build("my_ocaml_lib.exe.o"); // Or other .o target

    // Use a C/C++ build tool like `cc` to link the OCaml objects.
    let mut build = cc::Build::new();
    for object_path in objects {
        build.object(object_path);
    }
    build.compile("name_of_your_static_library"); // e.g., "libocaml_callable"
}
```

This `DuneBuilder` handles finding the dune project, constructing paths, and running the dune build command. The collected object files can then be passed to a tool like the `cc` crate to compile and link them into your Rust project.

## Configuration

The `DuneBuilder` can be configured using a fluent interface:

- **`new(ocaml_source_dir: P)`**: Creates a new builder. `ocaml_source_dir` is the path to your OCaml sources, relative to the `Cargo.toml` of the crate being built.
- **`with_profile(profile_name: &str)`**: Sets the Dune build profile (e.g., "release", "dev"). The default is "default".
- **`with_dune_args(extra_args: Vec<String>)`**: Appends custom arguments to the `dune build` command.
- **`with_dune_invocation(invocation: DuneInvocation)`**: Specifies how to call `dune`:
    - `DuneInvocation::OpamExec` (default): Uses `opam exec -- dune ...`.
    - `DuneInvocation::System`: Uses `dune ...` directly from the system path.
- **`build(target: T)`**: Executes the `dune build` command for the specified `target` (e.g., `my_lib.a`, `my_exe.exe.o`) and returns a `Vec<PathBuf>` of the compiled object files found in the build directory.
