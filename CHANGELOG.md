# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New userguide, replaces old documentation.
- New procedural macro `#[ocaml_interop::export]` in a dedicated `ocaml-interop-derive` crate,
  replaces the older `ocaml_export!` macro.
    * Introduces `OCaml<T>` (non-rooted) and `BoxRoot<T>` (auto-rooted) argument types,
      clarifying rooting strategy and replacing `OCamlRef<T>` for exported function arguments.
    * Direct mapping for Rust primitive types (`f64`, `i64`, `i32`, `bool`, `isize`) as arguments
      and return values, requiring corresponding `[@unboxed]` or `[@untagged]` attributes on the
      OCaml `external` declaration.
    * Automatic panic handling: Rust panics are caught and raised as OCaml exceptions (custom
      `RustPanic` or fallback `Failure`). Disable with `#[ocaml_interop::export(no_panic_catch)]`.
    * Bytecode wrapper generation via `#[ocaml_interop::export(bytecode = "stub_name")]`.
    * `noalloc` attribute: `#[ocaml_interop::export(noalloc)]` for functions that do not
      allocate on the OCaml heap. Requires `&OCamlRuntime` (immutable) and implies no panic catch.
      The OCaml `external` must be annotated with `[@@noalloc]`.

### Changed

- **Argument Handling in Exported Functions**: The `#[ocaml_interop::export]` macro promotes using
  `OCaml<T>` or `BoxRoot<T>` for OCaml value arguments instead of `OCamlRef<T>` to make rooting
  behavior explicit.

### Removed

- Removed the `ocaml_export!` macro.

## [0.11.2] - 2025-05-14

### Fixed

- Re-acquire the runtime lock before shutting down the OCaml runtime.

## [0.11.1] - 2025-05-07

### Fixed

- docs.rs documentation build.

## [0.11.0] - 2025-05-07

### Breaking Changes ⚠️

- **Dropped OCaml 4 Support**: This version exclusively supports OCaml 5.x. The underlying `ocaml-sys` dependency is now configured with the `ocaml5` feature, and the library leverages OCaml 5's domain-based concurrency model.
- **Runtime Management Overhaul for OCaml 5**:
    * `OCamlRuntime::init()` now returns `Result<OCamlRuntimeStartupGuard, String>`. The new `OCamlRuntimeStartupGuard` RAII type is responsible for OCaml runtime initialization (including `boxroot_setup`) and shutdown (calling `boxroot_teardown` and `caml_shutdown`).
    * The primary way to obtain an `&mut OCamlRuntime` is now via the new `OCamlRuntime::with_domain_lock(|cr| { ... })` method. This method handles OCaml domain registration and ensures the OCaml lock is held for the duration of the closure.
    * `OCamlRuntime` and `OCamlRuntimeStartupGuard` are now `!Send` and `!Sync` to reflect their thread-affinity with the OCaml runtime and domain state.
    * Removed `OCamlRuntime::init_persistent()` and the unsafe `OCamlRuntime::recover_handle()`.
- **`OCaml::nil()` Signature Change**: The static method `OCaml::nil()` now requires an `&mut OCamlRuntime` argument: `OCaml::nil(cr)`.
- **`BoxRoot<T>` Behavior**:
    * `BoxRoot::new()` and `BoxRoot::keep()` now panic if the underlying `boxroot_create` or `boxroot_modify` calls fail, providing an error message from `boxroot_error_string()`.
    * `BoxRoot<T>` is now explicitly `!Send` and `!Sync`.

### Added

- **OCaml 5 Support**: Full support for OCaml 5, including its domain-based concurrency model. This required significant internal changes to runtime and thread management.
- **`OCamlRuntimeStartupGuard`**: A new public RAII struct returned by `OCamlRuntime::init()`. Its `Drop` implementation handles the OCaml runtime shutdown.
- **`OCamlRuntime::with_domain_lock()`**: New static method to safely acquire a mutable reference to the OCaml runtime within a specific OCaml domain.

### Changed

- **Rust Edition**: Updated the crate's Rust edition from 2018 to 2021.
- **Internal Runtime Handling**: Functions exported to OCaml (via `ocaml_export!`) now use an updated internal mechanism (`crate::internal::recover_runtime_handle_mut()`) compatible with OCaml 5's per-domain runtime state.

### Removed

- **Dropped OCaml 4 Support**.
- `OCamlRuntime::init_persistent()` method.

## [0.10.0] - 2024-01-28

### Added

- Support for OCaml arrays (PR #61 by @mt-caret).
- Support for OCaml float arrays (PR #61 by @mt-caret).

### Changed

- OCaml exceptions are now represented by `OCaml<OCamlException>` values instead of custom `OCamlException` values. This makes them first-class and safer to use because the fact that executing the garbage collector can invalidate references is now encoded in the type (PR #60 by @mt-caret)

## [0.9.2] - 2023-07-27

### Fixed

- Expansion of bytecode-callable functions (PR #53, reported and debugged by @mt-caret in #52).

## [0.9.1] - 2023-07-12

### Fixed

- Expansion of bytecode-callable functions.

## [0.9.0] - 2023-07-12

### Added

- Support for OCaml `Bigarray.Array1` values (PR #26 by @g2p)
- Support for callable closure values with more automatic conversion between Rust and OCaml values (PR #44 and PR #45 by @sebastiencs)
- Conversions for `<Box<[u8]>` (PR #47 by @sebastiencs)
- Support for defining bytecode-callable wrappers for functions with `ocaml_export!` macro (PR #50)

## [0.8.8] - 2022-03-23

### Fixed

- Fixed compilation issue in some environments (PR #43 by @c-cube)

## [0.8.7] - 2021-10-12

### Added

- Conversion for tuples of 10 elements.

### Changed

- Internal changes to tuple conversion handling to use a slightly safer version of `store_field` to construct the tuples.

## [0.8.6] - 2021-10-12

### Fixed

- Bug in the conversion of `Result<T, E>` (Rust->OCaml) values that could result in memory corruption if the OCaml garbage collector happened to run during the conversion.

## [0.8.5] - 2021-08-16

### Added

- `no-caml-startup` feature flag that disables calls to `caml_startup` when initializing the runtime. This makes `OCamlRuntime::init_persistent()` a noop. It is useful for being able to load code that uses `ocaml-rs` in an utop toplevel (for example, when running `dune utop`). Will be enabled if the environment variable `OCAML_INTEROP_NO_CAML_STARTUP` is set. This is a temporary option that is likely to be removed in the future once a better solution is implemented.

## [0.8.4] - 2021-05-18

### Added

- Implementation of `ToOCaml<T>` for `BoxRoot<T>`.

### Added

- Conversion from/to unit (`()`) values.

## [0.8.3] - 2021-04-30

### Added

- Conversion from/to unit (`()`) values.

## [0.8.2] - 2021-04-27

### Fixed

- Breakage when building 0.8.1 on arm64 (by @zshipko).

## [0.8.1] - 2021-04-25

### Added

- `custom_ptr_val` method to `OCaml<T>` to obtain pointers to values embedded in OCaml custom blocks (by @g2p). Experimental API.
- `DynBox<T>`, support for custom OCaml values that wrap Rust values (by @g2p). Experimental API.

### Changed

- There is no longer a need to `ocaml_interop_setup` from OCaml programs.

## [0.8.0] - 2021-04-07

### Added

- Support for allocating OCaml polymorphic variants.
- `impl_to_ocaml_polymorphic_variant!` macro.
- `ocaml_alloc_polymorphic_variant!` macro.
- `caml_state` feature flag that gets forwarded to `ocaml-sys`.

## [0.7.2] - 2021-03-18

### Changed

- Bumped `ocaml-sys` dependency to at least `0.20.1`.

## [0.7.1] - 2021-03-18

### Changed

- Bumped `ocaml-boxroot-sys` dependency to `0.2` that includes a `without-ocamlopt` feature flag.

## [0.7.0] - 2021-03-16

### Removed

- `BoxRoot::from_raw` method.
- `BoxRoot::get_raw` method.

## [0.6.0] - 2021-03-15

### Added

- New BoxRoot type for rooted values, implemented by: https://gitlab.com/ocaml-rust/ocaml-boxroot
- `to_boxroot(cr)` method to `ToOCaml` trait.
- Support for OCaml tuples of up to 10 elements (until now tuples containing up to 4 elements were supported)
- `ocaml_interop_setup` and `ocaml_interop_teardown` functions that must be called by OCaml programs before calling Rust code.

### Changed

- Rooted values (both local and global roots) are now handled by BoxRoot.

### Removed

- `ocaml_frame!` macro. BoxRoot replaces the need for opening new local root frames.
- `to_ocaml!` macro. Without local roots this has no advantage for `value.to_ocaml(cr)`.


## [0.5.3] - 2021-01-26

### Security

- Added nul terminator to string passed to `caml_startup` (by @mat13mn)

## [0.5.2] - 2021-01-25

### Added

- `Drop` implementation for `OCamlRuntime` that shuts down the OCaml runtime.

### Changed

- `OCamlRuntime::recover_handle()` now returns a `&mut` reference instead of an owned value.

### Removed

- `OCamlRuntime::shutdown(self)`, now handled by `Drop` implementation.

## [0.5.1] - 2021-01-22

### Added

- Syntax in `impl_from_ocaml_variant!` for variant tags with named fields (records instead of tuples).

## [0.5.0] - 2021-01-20

### Added

- `OCamlRuntime::releasing_runtime(&mut self, f: FnOnce() -> T)` releases the OCaml runtime, calls `f`, and then re-acquires the OCaml runtime. Maybe more complicated patterns should be supported, but for now I haven't given this much thought.
- Support for unpacking OCaml polymorphic variants into Rust values.
- `to_rust(cr: &OCamlRuntime)` method to `OCamlRef<T>` values.
- `OCaml::unit()` method to obtain an OCaml unit value.
- `OCaml::none()` method to obtain an OCaml `None` value.
- Support for tuple-structs in struct/record mapping macros.
- `OCaml::as_ref(&self) -> OCamlRef<T>` method. Useful for immediate OCaml values (ints, unit, None, booleans) to convert them into `OCamlRef`s without the need for rooting.
- `Deref` implementation to get an `OCamlRef<T>` from an `OCaml<T>`.

### Changed

- The GC-handle has been replaced by an OCaml-Runtime-handle that must be passed around as a `&mut` reference. `OCaml<'a, T>` values have their lifetime associated to this handle through an immutable borrow, just like it used to be with the now-gone GC-handle.
- Exported functions don't implicitly open an `ocaml_frame!` anymore, but instead receive an OCaml-runtime-handle as their first argument.
- `ocaml_frame!` doesn't create a GC-handle anymore, but instead takes as input an OCaml-Runtime-handle, and uses it to instantiate a new frame and list of Root-variables through an immutable borrow.
- `ocaml_frame!` is now only required to instantiate Root-variables, any interaction with the OCaml runtime will make use of an OCaml-Runtime-handle, which should be around already. The syntax also changed slightly, requiring a comma after the OCaml-Runtime-handle parameter.
- `OCamlRoot` has been renamed to `OCamlRawRoot`.
- Rust functions that are exported to OCaml must now declare at least one argument.
- Functions that are exported to OCaml now follow a caller-save convention. These functions now receive `OCamlRef<T>`  arguments.
- Functions that are imported from OCaml now follow a caller-save convention. These functions  now receive `OCamlRef<T>`  arguments.
- Calls to OCaml functions don't return `Result` anymore, and instead panic on unexpected exceptions.
- `to_rust()` method is now implemented directly into `OCaml<T>`, the `ToRust` trait is not required anymore.
- `keep_raw()` method in root variables is now `unsafe` and returns an `OCamlRef<T>`.
- `OCaml<T>::as_i64()` -> `to_i64()`.
- `OCaml<T>::as_bool()` -> `to_bool()`.

### Removed

- `ToRust` trait.
- `OCamlRawRooted` type.
- `ocaml_call!` macro.
- `ocaml_alloc!` macro.
- `OCamlAllocToken` type.
- `OCamlRef::set` method (use `OCamlRawRoot::keep` instead).

## [0.4.4] - 2020-11-02

### Fixed

- Bug in macro when expanding root variables

## [0.4.3] - 2020-11-02

### Added

- Conversion to/from `Result` values.

## [0.4.2] - 2020-10-22

### Fixed

- Bug in `hd` and `tl` functions of `OCaml<OCamlList<A>>`.

## [0.4.1] - 2020-10-21

### Fixed

- docs.rs documentation build.

## [0.4.0] - 2020-10-20

### Changed

- This crate now depends on [ocaml-sys](https://crates.io/crates/ocaml-sys).
- Switched from `std` to `core` on every place it was possible.

## [0.3.0] - 2020-10-06

### Added

- Support for `Box<T>` conversions (boxed values get converted the same as their contents).
- `OCaml::of_i64_unchecked` as an `unsafe` unchecked version of `OCaml::of_i64`.
- More documentation.
- More tests.

### Changed

- `ocaml_frame!` and `ocaml_export!` macros now expect a list of local root variables. This fixes the hardcoded limit of 8 local roots per frame, and makes each frame allocate only as many roots as are actually needed.
- `to_ocaml!` now accepts an optional third argument. It must be a root variable. In this case an `OCamlRef<T>` is returned instead of an `OCaml<T>`.
- `IntoRust` and `into_rust()` have been renamed to `ToRust` and `to_rust()`.
- `OCaml::of_i64` can fail and return an error now (because of the possibly lost bit).
- Immutable borrows through `as_bytes` and `as_str` for `OCaml<String>` and `OCaml<OCamlBytes>` are no longer marked as `unsafe`.
- Made possible conversions between Rust `str`/`[u8]`/`String`/`Vec<u8>` values and `OCaml<OCamlBytes>` and `OCaml<String>` more explicit (used to be `AsRef<[u8]>` and `AsRef<str>` before).
- Add new `OCamlFloat` type for OCaml boxed floats, to more explicitly differentiate from unboxed float arguments in functions and declarations.

### Removed

- `nokeep` annotation when opening an `ocaml_frame!`/`ocaml_export!` function was removed. Opening the frame without any root variable declaration behaves the same as the old `nokeep` annotation.
- `OCaml<f64>` is no longer a valid representation for OCaml floats, use `OCaml<OCamlFloat>` instead.
- `keep` method in GC handles and `OCaml<T>` values was removed. The `keep` method in root variables should be used instead, or the third optional parameter of the `to_ocaml!` macro.

[Unreleased]: https://github.com/tizoc/ocaml-interop/compare/v0.11.2...HEAD
[0.11.2]: https://github.com/tizoc/ocaml-interop/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/tizoc/ocaml-interop/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/tizoc/ocaml-interop/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/tizoc/ocaml-interop/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/tizoc/ocaml-interop/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/tizoc/ocaml-interop/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/tizoc/ocaml-interop/compare/v0.8.8...v0.9.0
[0.8.8]: https://github.com/tizoc/ocaml-interop/compare/v0.8.7...v0.8.8
[0.8.7]: https://github.com/tizoc/ocaml-interop/compare/v0.8.6...v0.8.7
[0.8.6]: https://github.com/tizoc/ocaml-interop/compare/v0.8.5...v0.8.6
[0.8.5]: https://github.com/tizoc/ocaml-interop/compare/v0.8.4...v0.8.5
[0.8.4]: https://github.com/tizoc/ocaml-interop/compare/v0.8.3...v0.8.4
[0.8.3]: https://github.com/tizoc/ocaml-interop/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/tizoc/ocaml-interop/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/tizoc/ocaml-interop/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/tizoc/ocaml-interop/compare/v0.7.2...v0.8.0
[0.7.2]: https://github.com/tizoc/ocaml-interop/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/tizoc/ocaml-interop/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/tizoc/ocaml-interop/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/tizoc/ocaml-interop/compare/v0.5.3...v0.6.0
[0.5.3]: https://github.com/tizoc/ocaml-interop/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/tizoc/ocaml-interop/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/tizoc/ocaml-interop/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/tizoc/ocaml-interop/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/tizoc/ocaml-interop/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/tizoc/ocaml-interop/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/tizoc/ocaml-interop/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/tizoc/ocaml-interop/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/tizoc/ocaml-interop/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tizoc/ocaml-interop/compare/v0.2.4...v0.3.0
