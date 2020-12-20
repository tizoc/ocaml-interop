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
/// let arg1_rooted = root.keep(arg1);
/// # ()
/// });
/// ```
pub struct LivenessFailureCheck;

// Check that OCamlRoot values cannot escape the frame that created them.
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
pub struct OCamlRootEscapeFailureCheck;

// Check that OCamlRooted values cannot escape the frame that created the associated root.
// Must fail with:
// error[E0716]: temporary value dropped while borrowed
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let escaped = ocaml_frame!(cr, (rootvar), {
///     let arg1: OCaml<String> = "test".to_owned().to_ocaml(cr);
///     let arg1_rooted = rootvar.keep(arg1);
///     arg1_rooted
/// });
/// # ()
pub struct OCamlRootedEscapeFailureCheck;
