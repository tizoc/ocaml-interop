// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

// Must fail with:
// error[E0308]: mismatched types
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let arg1 = ocaml_alloc!(("test".to_owned()).to_ocaml(cr));
/// let result = ocaml_function(cr, arg1).unwrap();
/// # ()
/// ```
pub struct FailsWithoutOCamlCallMacro;

// Must fail with:
// error[E0308]: mismatched types
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let arg1 = ("test".to_owned()).to_ocaml(cr);
/// # ()
/// ```
pub struct FailsWithoutOCamlAllocMacro;

// Must fail with:
// error[E0502]: cannot borrow `*cr` as mutable because it is also borrowed as immutable
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let arg1 = ocaml_alloc!(("test".to_owned()).to_ocaml(cr));
/// let arg2 = ocaml_alloc!(("test".to_owned()).to_ocaml(cr));
/// let result = ocaml_call!(ocaml_function(cr, arg1)).unwrap();
/// let another_result = ocaml_call!(ocaml_function(cr, arg2)).unwrap();
/// # ()
/// ```
pub struct LivenessFailureCheck;

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

// Must fail with:
// error[E0716]: temporary value dropped while borrowed
/// ```compile_fail
/// # use ocaml_interop::*;
/// # ocaml! { pub fn ocaml_function(arg1: String) -> String; }
/// # let cr = &mut OCamlRuntime::init();
/// let escaped = ocaml_frame!(cr, (rootvar), {
///     let arg1 = ocaml_alloc!(("test".to_owned()).to_ocaml(cr));
///     let arg1_ref = rootvar.keep(arg1);
///     arg1_ref
/// });
/// # ()
pub struct OCamlRefEscapeFailureCheck;
