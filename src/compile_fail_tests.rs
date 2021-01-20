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

// Check that OCamlRef values cannot escape the frame that created the associated root.
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
/// let escaped = {
///     let ocaml_n: OCaml<'static, OCamlInt> = unsafe { OCaml::of_i64_unchecked(10) };
///     ocaml_n.as_value_ref()
/// };
/// # ()
/// ```
pub struct OCamlImmediateRootEscapeFailureCheck;

// Checks that OCamlRef values made from non-immediate OCaml values cannot be used
// as if they were references to rooted values.
// Must fail with:
// error[E0499]: cannot borrow `*cr` as mutable more than once at a time
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { fn ocaml_function(arg1: String) -> String; }
/// # fn test(cr: &'static mut OCamlRuntime) {
/// let arg1: OCaml<String> = to_ocaml!(cr, "test");
/// let _ = ocaml_function(cr, &arg1);
/// }
/// ```
pub struct NoStaticDerefsForNonImmediates;
