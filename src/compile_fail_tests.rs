// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

// Check that OCaml<T> values are not accessible after an allocation.
// Must fail with:
// error[E0499]: cannot borrow `*cr` as mutable more than once at a time
/// ```compile_fail
/// # use ocaml_interop::*;
/// # let cr = &mut OCamlRuntime::init();
/// ocaml_frame!(cr, (root), {
/// let arg1: OCaml<String> = "test".to_owned().to_ocaml(cr);
/// let arg2: OCaml<String> = "test".to_owned().to_ocaml(cr);
/// let arg1_root = root.keep(arg1);
/// # ()
/// });
/// ```
pub struct LivenessFailureCheck;

// Check that OCamlRawRoot values cannot escape the frame that created them.
// Must fail with:
// error[E0716]: temporary value dropped while borrowed
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let escaped = ocaml_frame!(cr, (rootvar), {
///     rootvar
/// });
/// # ()
/// ```
pub struct OCamlRawRootEscapeFailureCheck;

// Check that OCamlRoot values cannot escape the frame that created the associated root.
// Must fail with:
// error[E0716]: temporary value dropped while borrowed
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let escaped = ocaml_frame!(cr, (rootvar), {
///     let arg1: OCaml<String> = "test".to_owned().to_ocaml(cr);
///     let arg1_root = rootvar.keep(arg1);
///     arg1_root
/// });
/// # ()
/// ```
pub struct OCamlRootEscapeFailureCheck;

// Check that roots created from immediate values cannot escape.
// Must fail with:
// error[E0597]: `ocaml_n` does not live long enough
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let escaped = {
///     let ocaml_n: OCaml<'static, OCamlInt> = unsafe { OCaml::of_i64_unchecked(10) };
///     ocaml_n.as_root()
/// };
/// # ()
/// ```
pub struct OCamlImmediateRootEscapeFailureCheck;
