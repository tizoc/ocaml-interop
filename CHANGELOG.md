# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Support for `Box<T>` conversions (boxed values get converted the same as their contents).
- More documentation.
- More tests.

### Changed

- `ocaml_frame!` and `ocaml_export!` macros now expect a list of local root variables. This fixes the hardcoded limit of 8 local roots per frame, and makes each frame allocate only as many roots as are actually needed.
- Immutable borrows through `as_bytes` and `as_str` for `OCaml<String>` and `OCaml<OCamlBytes>` are no longer marked as `unsafe`.
- Made possible conversions between Rust `str`/`[u8]`/`String`/`Vec<u8>` values and `OCaml<OCamlBytes>` and `OCaml<String>` more explicit (used to be `AsRef<[u8]>` and `AsRef<str>` before).
- Add new `OCamlFloat` type for OCaml boxed floats, to more explicitly differentiate from unboxed float arguments in functions and declarations.

### Deprecated

- Nothing.

### Removed

- `OCaml<f64>` is no longer a valid representation for OCaml floats, use `OCaml<OCamlFloat>` instead.

### Fixed

- Nothing.

### Security

- Nothing.

[Unreleased]: https://github.com/simplestaking/ocaml-interop/compare/v0.2.4...HEAD
