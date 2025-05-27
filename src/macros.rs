// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

#[cfg(doc)]
use crate::*;

/// Declares OCaml functions.
///
/// `ocaml! { pub fn registered_name(arg1: ArgT, ...) -> Ret_typ; ... }` declares a function that has been
/// defined in OCaml code and registered with `Callback.register "registered_name" ocaml_function`.
///
/// Visibility and return value type can be omitted. The return type defaults to `()` when omitted.
///
/// When invoking one of these functions, the first argument must be a `&mut `[`OCamlRuntime`],
/// and the remaining arguments [`OCamlRef`]`<ArgT>`.
///
/// The return value is a [`BoxRoot`]`<RetType>`.
///
/// Calls that raise an OCaml exception will `panic!`. Care must be taken on the OCaml side
/// to avoid exceptions and return `('a, 'err) Result.t` values to signal errors, which
/// can then be converted into Rust's `Result<A, Err>` and `Result<OCaml<A>, OCaml<Err>>`.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # struct MyRecord {};
/// ocaml! {
///     // Declares `print_endline`, with a single `String` (`OCamlRef<String>` when invoked)
///     // argument and `BoxRoot<()>` return type (default when omitted).
///     pub fn print_endline(s: String);
///
///     // Declares `bytes_concat`, with two arguments, an OCaml `bytes` separator,
///     // and an OCaml list of segments to concatenate. Return value is an OCaml `bytes`
///     // value.
///     fn bytes_concat(sep: OCamlBytes, segments: OCamlList<OCamlBytes>) -> OCamlBytes;
/// }
/// ```
#[macro_export]
macro_rules! ocaml {
    () => ();

    ($vis:vis fn $name:ident(
        $arg:ident: $typ:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a, 'b: 'a, RustT>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg: impl $crate::OCamlParam<'a, 'b, RustT, $typ>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            let rooted_arg = $arg.to_rooted(cr);
            let ocaml_ref = match rooted_arg {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            $crate::BoxRoot::new(closure.call(cr, ocaml_ref))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a, 'b: 'a, RustT1, RustT2>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg1: impl $crate::OCamlParam<'a, 'b, RustT1, $typ1>,
            $arg2: impl $crate::OCamlParam<'a, 'b, RustT2, $typ2>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            let rooted_arg1 = $arg1.to_rooted(cr);
            let rooted_arg2 = $arg2.to_rooted(cr);
            let ocaml_ref1 = match rooted_arg1 {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            let ocaml_ref2 = match rooted_arg2 {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            $crate::BoxRoot::new(closure.call2(cr, ocaml_ref1, ocaml_ref2))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty,
        $arg3:ident: $typ3:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a, 'b: 'a, RustT1, RustT2, RustT3>(
            cr: &'a mut $crate::OCamlRuntime,
            $arg1: impl $crate::OCamlParam<'a, 'b, RustT1, $typ1>,
            $arg2: impl $crate::OCamlParam<'a, 'b, RustT2, $typ2>,
            $arg3: impl $crate::OCamlParam<'a, 'b, RustT3, $typ3>,
        ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            let rooted_arg1 = $arg1.to_rooted(cr);
            let rooted_arg2 = $arg2.to_rooted(cr);
            let rooted_arg3 = $arg3.to_rooted(cr);
            let ocaml_ref1 = match rooted_arg1 {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            let ocaml_ref2 = match rooted_arg2 {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            let ocaml_ref3 = match rooted_arg3 {
                $crate::RefOrRooted::Ref(r) => r,
                $crate::RefOrRooted::Root(ref root) => &**root,
            };
            $crate::BoxRoot::new(closure.call3(cr, ocaml_ref1, ocaml_ref2, ocaml_ref3))
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $($arg:ident: $typ:ty),+ $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis fn $name<'a>(
            cr: &'a mut $crate::OCamlRuntime,
            $($arg: $crate::OCamlRef<$typ>),+
    ) -> $crate::BoxRoot<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(closure, $name);
            $crate::BoxRoot::new(closure.call_n(cr, &mut [$(unsafe { $arg.get_raw() }),+]))
        }

        $crate::ocaml!($($t)*);
    }
}

// Internal utility macros

#[doc(hidden)]
#[macro_export]
macro_rules! count_fields {
    () => {0usize};
    ($_f1:ident $_f2:ident $_f3:ident $_f4:ident $_f5:ident $($fields:ident)*) => {
        5usize + $crate::count_fields!($($fields)*)
    };
    ($field:ident $($fields:ident)*) => {1usize + $crate::count_fields!($($fields)*)};
}

#[doc(hidden)]
#[macro_export]
macro_rules! ocaml_closure_reference {
    ($var:ident, $name:ident) => {
        static NAME: &str = stringify!($name);
        static mut OC: Option<$crate::internal::OCamlClosure> = None;
        static INIT: ::std::sync::Once = ::std::sync::Once::new();
        let $var = unsafe {
            INIT.call_once(|| {
                OC = $crate::internal::OCamlClosure::named(NAME);
            });
            OC.unwrap_or_else(|| panic!("OCaml closure with name '{}' not registered", NAME))
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_to_unit {
    // No return value, default to unit
    () => {
        ()
    };

    // Return value specified
    ($rtyp:ty) => {
        $rtyp
    };
}
