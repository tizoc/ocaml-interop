# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/simplestaking/ocaml-interop/compare/v0.5.3...HEAD
[0.5.3]: https://github.com/simplestaking/ocaml-interop/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/simplestaking/ocaml-interop/compare/v0.5.1...v0.5.2
[0.5.1]: https://github.com/simplestaking/ocaml-interop/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/simplestaking/ocaml-interop/compare/v0.4.4...v0.5.0
[0.4.4]: https://github.com/simplestaking/ocaml-interop/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/simplestaking/ocaml-interop/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/simplestaking/ocaml-interop/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/simplestaking/ocaml-interop/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/simplestaking/ocaml-interop/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/simplestaking/ocaml-interop/compare/v0.2.4...v0.3.0
