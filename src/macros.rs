// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

/// Opens a new frame inside of which new OCaml values can be allocated and OCaml functions called.
///
/// The first argument to this macro must be an identifier to which the OCaml frame GC handle will be bound.
/// It will be used to allocate OCaml values and call OCaml functions. Value references that result from allocations
/// or function calls, will have their lifetime bound to this handle.
///
/// Optionally, this identifier can be followed by a list of names to which "keep variables" will be bound.
/// For each one of these variables, a new location for a tracked pointer will be reserved. Each one of these
/// "keep variables" can be consumed to produce an `OCamlRef` that will be used to re-reference OCaml values
/// what would otherwise be unavailable after an OCaml allocation of OCaml function call.
///
/// # Notes
///
/// When no "keep variables" are declared when opening a frame `ocaml-interop` will avoid setting up a new
/// local roots frame, because it is not necessary in that case.
///
/// # Examples
///
/// The following example reserves space for two tracked pointers:
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! {
/// #    fn print_endline(s: String);
/// # }
/// # fn ocaml_frame_macro_example() {
///     ocaml_frame!(gc(hello_ocaml, bye_ocaml), { // `gc` gets bound to the frame handle
///         let hello_ocaml = &to_ocaml!(gc, "hello OCaml!", hello_ocaml);
///         let bye_ocaml = &to_ocaml!(gc, "bye OCaml!", bye_ocaml);
///         ocaml_call!(print_endline(gc, gc.get(hello_ocaml)));
///         ocaml_call!(print_endline(gc, gc.get(bye_ocaml)));
///         // Values that don't need to be keept across calls can be used directly
///         let immediate_use = to_ocaml!(gc, "no need to `keep` me");
///         ocaml_call!(print_endline(gc, immediate_use));
///     });
/// # }
/// ```
///
/// The following example does not declare "keep variables". As a result no space is reserved for local roots:
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn print_endline(s: String); }
/// # fn ocaml_frame_macro_example() {
///     ocaml_frame!(gc, {
///         let ocaml_string = to_ocaml!(gc, "hello OCaml!");
///         ocaml_call!(print_endline(gc, ocaml_string));
///     });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_frame {
    ($gc:ident, $body:block) => {{
        let mut frame: $crate::internal::GCFrameNoKeep = Default::default();
        let $gc = frame.initialize();
        $body
    }};

    ($gc:ident($($keeper:ident),+ $(,)?), $body:block) => {{
        let mut frame: $crate::internal::GCFrame = Default::default();
        let local_roots = $crate::repeat_slice!(::std::cell::Cell::new($crate::UNIT), $($keeper)+);
        let $gc = frame.initialize(&local_roots);
        $(
            let mut $keeper = unsafe { $crate::internal::KeepVar::reserve(&$gc) };
        )+
        $body
    }};

    ($($t:tt)*) => {
        compile_error!("Invalid `ocaml_frame!` syntax. Must be `ocaml_frame! { gc | gc(vars, ...), { body-block } }`.")
    };
}

/// Declares OCaml functions.
///
/// `ocaml! { pub fn ocaml_name(arg1: typ1, ...) -> ret_typ; ... }` declares a function that has been
/// defined in OCaml code and registered with `Callback.register "ocaml_name" the_function`.
///
/// Visibility and return value type can be omitted. The return type defaults to unit when omitted.
///
/// These functions must be invoked with the `ocaml_call!` macro.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # struct MyRecord {};
/// ocaml! {
///     // Declares `print_endline`, with a single `String` (`OCaml<String>` when invoked)
///     // argument and unit return type (default when omitted).
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
        $vis unsafe fn $name(
            token: $crate::OCamlAllocToken,
            $arg: $crate::OCaml<$typ>,
        ) -> $crate::OCamlResult<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call(token, $arg)
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(
            token: $crate::OCamlAllocToken,
            $arg1: $crate::OCaml<$typ1>,
            $arg2: $crate::OCaml<$typ2>,
        ) -> $crate::OCamlResult<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call2(token, $arg1, $arg2)
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $arg1:ident: $typ1:ty,
        $arg2:ident: $typ2:ty,
        $arg3:ident: $typ3:ty $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(
            token: $crate::OCamlAllocToken,
            $arg1: $crate::OCaml<$typ1>,
            $arg2: $crate::OCaml<$typ2>,
            $arg3: $crate::OCaml<$typ3>,
        ) -> $crate::OCamlResult<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call3(token, $arg1, $arg2, $arg3)
        }

        $crate::ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident(
        $($arg:ident: $typ:ty),+ $(,)?
    ) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(
            token: $crate::OCamlAllocToken,
            $($arg: $crate::OCaml<$typ>),+
    ) -> $crate::OCamlResult<$crate::default_to_unit!($($rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call_n(token, &mut [$($arg.raw()),+])
        }

        $crate::ocaml!($($t)*);
    }
}

/// Defines Rust functions callable from OCaml.
///
/// The first argument in these functions declarations is the same as in the [`ocaml_frame!`] macro.
///
/// Arguments and return values must be of type `OCaml<T>`, or `f64` in the case of unboxed floats.
///
/// The return type defaults to unit when omitted.
///
/// The body of the function has an implicit `ocaml_frame!` wrapper, with the lifetimes of every `OCaml<T>`
/// argument bound to the lifetime of the variable bound to the function's OCaml frame GC handle.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// ocaml_export! {
///     fn rust_twice(_gc, num: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
///         let num: i64 = num.into_rust();
///         unsafe { OCaml::of_i64(num * 2) }
///     }
///
///     fn rust_twice_boxed_i32(gc, num: OCaml<OCamlInt32>) -> OCaml<OCamlInt32> {
///         let num: i32 = num.into_rust();
///         let result = num * 2;
///         ocaml_alloc!(result.to_ocaml(gc))
///     }
///
///     fn rust_add_unboxed_floats_noalloc(_gc, num: f64, num2: f64) -> f64 {
///         num * num2
///     }
///
///     fn rust_twice_boxed_float(gc, num: OCaml<OCamlFloat>) -> OCaml<OCamlFloat> {
///         let num: f64 = num.into_rust();
///         let result = num * 2.0;
///         ocaml_alloc!(result.to_ocaml(gc))
///     }
///
///     fn rust_increment_ints_list(gc, ints: OCaml<OCamlList<OCamlInt>>) -> OCaml<OCamlList<OCamlInt>> {
///         let mut vec: Vec<i64> = ints.into_rust();
///
///         for i in 0..vec.len() {
///             vec[i] += 1;
///         }
///
///         ocaml_alloc!(vec.to_ocaml(gc))
///     }
///
///     fn rust_make_tuple(gc, fst: OCaml<String>, snd: OCaml<OCamlInt>) -> OCaml<(String, OCamlInt)> {
///         let fst: String = fst.into_rust();
///         let snd: i64 = snd.into_rust();
///         let tuple = (fst, snd);
///         ocaml_alloc!(tuple.to_ocaml(gc))
///     }
/// }
/// ```
///
/// [`ocaml_frame!`]: ./macro.ocaml_frame.html
#[macro_export]
macro_rules! ocaml_export {
    {} => ();

    // Unboxed float return
    {
        fn $name:ident( $gc:ident $(($($keeper:ident),+ $(,)?))?, $($args:tt)*) -> f64
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function!(
            @name $name
            @gc { $gc $(($($keeper),+))? }
            @final_args { }
            @proc_args { $($args)*, }
            @return { f64 }
            @body $body
            @original_args $($args)*
        );

        $crate::ocaml_export!{$($t)*}
    };

    // Other (or empty) return value type
    {
        fn $name:ident( $gc:ident $(($($keeper:ident),+ $(,)?))?, $($args:tt)*) $(-> $rtyp:ty)?
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function!(
            @name $name
            @gc { $gc $(($($keeper),+))? }
            @final_args { }
            @proc_args { $($args)*, }
            @return { $($rtyp)? }
            @body $body
            @original_args $($args)*
        );

        $crate::ocaml_export!{$($t)*}
    };
}

/// Calls an OCaml allocator function.
///
/// Useful for calling functions that construct new values and never raise an exception.
///
/// It is used internally by the `to_ocaml` macro, and may be used directly only in rare occasions.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # fn to_ocaml_macro_example() {
///     let hello_string = "hello OCaml!";
///     ocaml_frame!(gc, {
///         let ocaml_string: OCaml<String> = ocaml_alloc!(hello_string.to_ocaml(gc));
///         // ...
///     });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_alloc {
    ( $(($obj:expr).)?$($fn:ident).+($gc:ident $(,)?) ) => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() });
            res.mark($gc).eval($gc)
        }
    };

    ( $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),+ $(,)? ) ) => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() }, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };

    ( $obj:literal.$($fn:ident).+($gc:ident $(,)?) ) => {
        {
            let res = $obj.$($fn).+(unsafe { $gc.token() });
            res.mark($gc).eval($gc)
        }
    };

    ( $obj:literal.$($fn:ident).+($gc:ident, $($arg:expr),+ $(,)?) ) => {
        {
            let res = $obj.$($fn).+(unsafe { $gc.token() }, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };
}

/// Converts Rust values into OCaml values.
///
/// In `to_ocaml!(gc, value)`, `gc` is an OCaml frame GC handle, and `value` is
/// a Rust value of a type that implements the `ToOCaml` trait. The resulting
/// value's lifetime is bound to `gc`'s.
///
/// An alternative form accepts a third "keep variable" argument: `to_ocaml!(gc, value, keepvar)`.
/// `keepvar` is one of the variables declared when opening an `ocaml_frame!`.
/// This variant consumes `keepvar` returns an `OCamlRef` value instead of an `OCaml` one.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # fn to_ocaml_macro_example() {
///     ocaml_frame!(gc, {
///         let ocaml_string: OCaml<String> = to_ocaml!(gc, "hello OCaml!");
///         // ...
///         # ()
///     });
/// # }
/// ```
///
/// Variant:
///
/// ```
/// # use ocaml_interop::*;
/// # fn to_ocaml_macro_example() {
///     ocaml_frame!(gc(keepvar), {
///         let ocaml_string_ref: &OCamlRef<String> = &to_ocaml!(gc, "hello OCaml!", keepvar);
///         // ...
///         # ()
///     });
/// # }
/// ```
#[macro_export]
macro_rules! to_ocaml {
    ($gc:ident, $obj:expr, $keepvar:ident) => {
        $keepvar.keep($crate::to_ocaml!($gc, $obj))
    };

    ($gc:ident, $obj:expr) => {
        $crate::ocaml_alloc!(($obj).to_ocaml($gc))
    };

    ($($t:tt)*) => {
        compile_error!("Incorrect `to_ocaml!` syntax. Must be `to_ocaml!(gc, expr[, keepvar])`")
    };
}

/// Calls an OCaml function
///
/// The called function must be declared with `ocaml!`. This macro can
/// only be used inside `ocaml_frame!` blocks, and the framce GC
/// handle must be passed as the first argument to the function.
///
/// The result is either `Ok(result)` or `Err(ocaml_exception)` if
/// an exception is raised by the OCaml function.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// ocaml! { fn print_endline(s: String); }
///
/// # fn ocaml_frame_macro_example() {
/// // ...somewhere else inside a function
/// ocaml_frame!(gc, {
///     let ocaml_string = to_ocaml!(gc, "hello OCaml!");
///     ocaml_call!(print_endline(gc, ocaml_string)).unwrap();
/// });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_call {
    ( $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),+ $(,)?)) => {
        {
            let res = unsafe { $(($obj).)?$($fn).+($gc.token(), $($arg),* ) };
            $crate::gcmark_result!($gc, res)
        }
    };

    ( $($path:ident)::+($gc:ident, $($args:expr),+ $(,)?) ) => {
        {
            let res = unsafe { $($path)::+($gc.token(), $($args),+) };
            $crate::gcmark_result!($gc, res)
        }
    };

    ( $($path:ident)::+.$($field:ident).+($gc:ident, $($args:expr),+ $(,)?) ) => {
        {
            let res = unsafe { $($path)::+$($field).+($gc.token(), $($args),+) };
            $crate::gcmark_result!($gc, res)
        }
    };
}

/// Implements conversion between a Rust struct and an OCaml record.
///
/// See the `impl_to_ocaml_record!` and `impl_from_ocaml_record!` macros
/// for more details.
#[macro_export]
macro_rules! impl_conv_ocaml_record {
    ($rust_typ:ident => $ocaml_typ:ident {
        $($field:ident : $ocaml_field_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        $crate::impl_to_ocaml_record! {
            $rust_typ => $ocaml_typ {
                $($field : $ocaml_field_typ $(=> $conv_expr)?),+
            }
        }

        $crate::impl_from_ocaml_record! {
            $ocaml_typ => $rust_typ {
                $($field : $ocaml_field_typ),+
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_conv_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements conversion between a Rust enum and an OCaml variant.
///
/// See the `impl_to_ocaml_variant!` and `impl_from_ocaml_variant!` macros
/// for more details.
#[macro_export]
macro_rules! impl_conv_ocaml_variant {
    ($rust_typ:ty => $ocaml_typ:ty {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(=> $conv:expr)?),+ $(,)?
    }) => {
        $crate::impl_to_ocaml_variant! {
            $rust_typ => $ocaml_typ {
                $($($tag)::+ $(($($slot_name: $slot_typ),+))? $(=> $conv)?),+
            }
        }

        $crate::impl_from_ocaml_variant! {
            $ocaml_typ => $rust_typ {
                $($($tag)::+ $(($($slot_name: $slot_typ),+))?),+
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_conv_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Unpacks an OCaml record into a Rust record
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_mystruct(unit: ()) -> MyStruct; }
/// struct MyStruct {
///     int_field: i64,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// # fn unpack_record_example() {
/// #   ocaml_frame!(gc, {
/// let ocaml_struct = ocaml_call!(make_mystruct(gc, OCaml::unit())).unwrap();
/// ocaml_unpack_record! {
///     //  value    => RustConstructor { field: OCamlType, ... }
///     ocaml_struct => MyStruct {
///         int_field: OCamlInt,
///         string_field: String,
///     }
/// }
/// #   });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_unpack_record {
    ($var:ident => $cons:ident {
        $($field:ident : $ocaml_typ:ty),+ $(,)?
    }) => {
        let record = $var;
        unsafe {
            let mut current = 0;

            $(
                let $field = record.field::<$ocaml_typ>(current).into_rust();
                current += 1;
            )+

            $cons {
                $($field),+
            }
        }
    };
}

/// Allocates an OCaml memory block tagged with the specified value.
///
/// It is used internally to allocate OCaml variants, its direct use is
/// not recommended.
#[macro_export]
macro_rules! ocaml_alloc_tagged_block {
    ($tag:expr, $($field:ident : $ocaml_typ:ty),+ $(,)?) => {
        unsafe {
            $crate::ocaml_frame!(gc(block), {
                let mut current = 0;
                let field_count = $crate::count_fields!($($field)*);
                let block = block.keep_raw($crate::internal::caml_alloc(field_count, $tag));
                $(
                    let $field: $crate::OCaml<$ocaml_typ> = $crate::to_ocaml!(gc, $field);
                    $crate::internal::store_field(block.get_raw(), current, $field.raw());
                    current += 1;
                )+
                $crate::OCamlAllocResult::of(block.get_raw())
            })
        }
    };
}

/// Allocates an OCaml record built from a Rust record
///
/// Most of the `impl_to_ocaml_record!` macro will be used to define how records
/// should be converted. This macro is useful when implementing OCaml allocation
/// functions directly.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// struct MyStruct {
///     int_field: u8,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// # fn alloc_record_example() {
/// #   ocaml_frame!(gc, {
/// let ms = MyStruct { int_field: 132, string_field: "blah".to_owned() };
/// let ocaml_ms: OCamlAllocResult<MyStruct> = ocaml_alloc_record! {
///     //  value { field: OCamlType, ... }
///     ms {
///         // optionally `=> expr` can be used to preprocess the field value
///         // before the conversion into OCaml takes place.
///         // Inside the expression, a variable with the same name as the field
///         // is bound to a reference to the field value.
///         int_field: OCamlInt => { *int_field as i64 },
///         string_field: String,
///     }
/// };
/// // ...
/// # ()
/// #   });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_alloc_record {
    ($self:ident {
        $($field:ident : $ocaml_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe {
            $crate::ocaml_frame!(gc(record), {
                let mut current = 0;
                let field_count = $crate::count_fields!($($field)*);
                let record = record.keep_raw($crate::internal::caml_alloc(field_count, 0));
                $(
                    let $field = &$crate::prepare_field_for_mapping!($self.$field $(=> $conv_expr)?);
                    let $field: $crate::OCaml<$ocaml_typ> = $crate::to_ocaml!(gc, $field);
                    $crate::internal::store_field(record.get_raw(), current, $field.raw());
                    current += 1;
                )+
                $crate::OCamlAllocResult::of(record.get_raw())
            })
        }
    };
}

/// Implements `FromOCaml` for mapping an OCaml record into a Rust record.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_mystruct(unit: ()) -> MyStruct; }
/// struct MyStruct {
///     int_field: i64,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// impl_from_ocaml_record! {
///     // Optionally, if Rust and OCaml types don't match:
///     // OCamlType => RustType { ... }
///     MyStruct {
///         int_field: OCamlInt,
///         string_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_ocaml_record {
    ($ocaml_typ:ident => $rust_typ:ident {
        $($field:ident : $ocaml_field_typ:ty),+ $(,)?
    }) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                $crate::ocaml_unpack_record! { v =>
                    $rust_typ {
                        $($field : $ocaml_field_typ),+
                    }
                }
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_from_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements `ToOCaml` for mapping a Rust record into an OCaml record.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// struct MyStruct {
///     int_field: u8,
///     string_field: String,
/// }
///
/// // Assuming an OCaml record declaration like:
/// //
/// //      type my_struct = {
/// //          int_field: int;
/// //          string_field: string;
/// //      }
/// //
/// // NOTE: What is important is the order of the fields, not their names.
///
/// impl_to_ocaml_record! {
///     // Optionally, if Rust and OCaml types don't match:
///     // RustType => OCamlType { ... }
///     MyStruct {
///         // optionally `=> expr` can be used to preprocess the field value
///         // before the conversion into OCaml takes place.
///         // Inside the expression, a variable with the same name as the field
///         // is bound to a reference to the field value.
///         int_field: OCamlInt => { *int_field as i64 },
///         string_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_to_ocaml_record {
    ($rust_typ:ty => $ocaml_typ:ident {
        $($field:ident : $ocaml_field_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe impl $crate::ToOCaml<$ocaml_typ> for $rust_typ {
            fn to_ocaml(&self, _token: $crate::OCamlAllocToken) -> $crate::OCamlAllocResult<$ocaml_typ> {
                $crate::ocaml_alloc_record! {
                    self {
                        $($field : $ocaml_field_typ $(=> $conv_expr)?),+
                    }
                }
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        $crate::impl_to_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Implements `FromOCaml` for mapping an OCaml variant into a Rust enum.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// impl_from_ocaml_variant! {
///     // Optionally, if Rust and OCaml types don't match:
///     // OCamlType => RustType { ... }
///     Movement {
///         // Alternative: StepLeft  => Movement::StepLeft
///         //              <anyname> => <build-expr>
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_ocaml_variant {
    ($ocaml_typ:ty => $rust_typ:ty {
        $($t:tt)*
    }) => {
        unsafe impl $crate::FromOCaml<$ocaml_typ> for $rust_typ {
            fn from_ocaml(v: $crate::OCaml<$ocaml_typ>) -> Self {
                let result = $crate::ocaml_unpack_variant! {
                    v => {
                        $($t)*
                    }
                };

                let msg = concat!(
                    "Failure when unpacking an OCaml<", stringify!($ocaml_typ), "> variant into ",
                    stringify!($rust_typ), " (unexpected tag value)");

                result.expect(msg)
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_from_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

/// Unpacks an OCaml variant and maps it into a Rust enum.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Note
///
/// Unlike with `ocaml_unpack_record!`, the result of `ocaml_unpack_variant!` is a `Result` value.
/// An error will be returned in the case of an expected tag value. This may change in the future.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_ocaml_movement(unit: ()) -> Movement; }
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type my_struct =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// # fn unpack_variant_example() {
/// #   ocaml_frame!(gc, {
/// let ocaml_variant = ocaml_call!(make_ocaml_movement(gc, OCaml::unit())).unwrap();
/// let result = ocaml_unpack_variant! {
///     ocaml_variant => {
///         // Alternative: StepLeft  => Movement::StepLeft
///         //              <anyname> => <build-expr>
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }.unwrap();
/// // ...
/// #   });
/// # }
#[macro_export]
macro_rules! ocaml_unpack_variant {
    ($self:ident => {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(=> $conv:expr)?),+ $(,)?
    }) => {
        (|| {
            let mut current_block_tag = 0;
            let mut current_long_tag = 0;

            $(
                $crate::unpack_variant_tag!(
                    $self, current_block_tag, current_long_tag,
                    $($tag)::+ $(($($slot_name: $slot_typ),+))? $(=> $conv)?);
            )+

            Err("Invalid tag value found when converting from an OCaml variant")
        })()
    };
}

/// Allocates an OCaml variant, mapped from a Rust enum.
///
/// The conversion is exhaustive, and requires that every enum case is handled.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// # ocaml! { fn make_ocaml_movement(unit: ()) -> Movement; }
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// # fn alloc_variant_example() {
/// #   ocaml_frame!(gc, {
/// let movement = Movement::Rotate(180.0);
/// let ocaml_movement: OCamlAllocResult<Movement> = ocaml_alloc_variant! {
///     movement => {
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// };
/// // ...
/// #   });
/// # }
/// ```
#[macro_export]
macro_rules! ocaml_alloc_variant {
    ($self:ident => {
        $($($tag:ident)::+ $(($($slot_name:ident: $slot_typ:ty),+ $(,)?))? $(,)?),+
    }) => {
        $crate::ocaml_alloc_variant_match!{
            $self, 0u8, 0u8,

            @units {}
            @blocks {}

            @pending $({ $($tag)::+ $(($($slot_name: $slot_typ),+))? })+
        }
    };
}

/// Implements `ToOCaml` for mapping a Rust enum into an OCaml variant.
///
/// The conversion is exhaustive, and requires that every enum case is handled.
///
/// It is important that the order remains the same as in the OCaml type declaration.
///
/// # Examples
///
/// # Examples
///
/// ```
/// # use ocaml_interop::*;
/// enum Movement {
///     StepLeft,
///     StepRight,
///     Rotate(f64),
/// }
///
/// // Assuming an OCaml type declaration like:
/// //
/// //      type movement =
/// //        | StepLeft
/// //        | StepRight
/// //        | Rotate of float
/// //
/// // NOTE: What is important is the order of the tags, not their names.
///
/// impl_to_ocaml_variant! {
///     // Optionally, if Rust and OCaml types don't match:
///     // RustType => OCamlType { ... }
///     Movement {
///         Movement::StepLeft,
///         Movement::StepRight,
///         // Tag field names are mandatory
///         Movement::Rotate(rotation: OCamlFloat),
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_to_ocaml_variant {
    ($rust_typ:ty => $ocaml_typ:ty {
        $($t:tt)*
    }) => {
        unsafe impl $crate::ToOCaml<$ocaml_typ> for $rust_typ {
            fn to_ocaml(&self, _token: $crate::OCamlAllocToken) -> $crate::OCamlAllocResult<$ocaml_typ> {
                $crate::ocaml_alloc_variant! {
                    self => {
                        $($t)*
                    }
                }
            }
        }
    };

    ($both_typ:ty {
        $($t:tt)*
    }) => {
        $crate::impl_to_ocaml_variant!{
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

// Internal utility macros

#[doc(hidden)]
#[macro_export]
macro_rules! repeat_slice {
    (@expr $value:expr;
     @accum [$($accum:expr),+];
     @rest) => {
         [$($accum),+]
     };

    (@expr $value:expr;
     @accum [$($accum:expr),+];
     @rest $_v1:ident $_v2:ident $_v3:ident $_v4:ident $_v5:ident $($vars:ident)*) => {
        $crate::repeat_slice!(
            @expr $value;
            @accum [$value, $value, $value, $value, $value, $($accum),+];
            @rest $vars)
    };

    (@expr $value:expr;
        @accum [$($accum:expr),+];
        @rest $_v1:ident $($vars:ident)*) => {

        $crate::repeat_slice!(
            @expr $value;
            @accum [$value, $($accum),+];
            @rest $($vars)*)
    };

    ($value:expr, $field:ident $($vars:ident)*) => {
        $crate::repeat_slice!(
            @expr $value;
            @accum [$value];
            @rest $($vars)*)
    };
}


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
macro_rules! prepare_field_for_mapping {
    ($self:ident.$field:ident) => {
        $self.$field
    };

    ($self:ident.$field:ident => $conv_expr:expr) => {{
        let $field = &$self.$field;
        $conv_expr
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! unpack_variant_tag {
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+) => {
        $crate::unpack_variant_tag!($self, $current_block_tag, $current_long_tag, $($tag)::+ => $($tag)::+)
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+ => $conv:expr) => {
        if $self.is_long() && unsafe { $crate::internal::raw_ocaml_to_i64($self.raw()) } == $current_long_tag {
            return Ok($conv);
        }
        $current_long_tag += 1;
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+)) => {

        $crate::unpack_variant_tag!(
            $self, $current_block_tag, $current_long_tag,
            $($tag)::+ ($($slot_name: $slot_typ),+) => $($tag)::+($($slot_name),+))
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident,
        $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+) => $conv:expr) => {

        if $self.is_block() && $self.tag_value() == $current_block_tag {
            let mut current_field = 0;

            $(
                let $slot_name = unsafe { $self.field::<$slot_typ>(current_field).into_rust() };
                current_field += 1;
            )+

            return Ok($conv);
        }
        $current_block_tag += 1;
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ocaml_alloc_variant_match {
    // Base case, generate `match` expression
    ($self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units {
            $({ $($unit_tag:ident)::+ @ $unit_tag_counter:expr })*
        }
        @blocks {
            $({ $($block_tag:ident)::+ ($($block_slot_name:ident: $block_slot_typ:ty),+) @ $block_tag_counter:expr })*
        }

        @pending
    ) => {
        match $self {
            $(
                $($unit_tag)::+ =>
                    $crate::OCamlAllocResult::of(unsafe { $crate::OCaml::of_i64($unit_tag_counter as i64).raw() }),
            )*
            $(
                $($block_tag)::+($($block_slot_name),+) =>
                    $crate::ocaml_alloc_tagged_block!($block_tag_counter, $($block_slot_name: $block_slot_typ),+),
            )*
        }
    };

    // Found unit tag, add to accumulator and increment unit variant tag number
    ($self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units { $($unit_tags_accum:tt)* }
        @blocks { $($block_tags_accum:tt)* }

        @pending
            { $($found_tag:ident)::+ }
            $($tail:tt)*
    ) => {
        $crate::ocaml_alloc_variant_match!{
            $self, $current_block_tag, {1u8 + $current_long_tag},

            @units {
                $($unit_tags_accum)*
                { $($found_tag)::+ @ $current_long_tag }
            }
            @blocks { $($block_tags_accum)* }

            @pending $($tail)*
        }
    };

    // Found block tag, add to accumulator and increment block variant tag number
    ($self:ident, $current_block_tag:expr, $current_long_tag:expr,

        @units { $($unit_tags_accum:tt)* }
        @blocks { $($block_tags_accum:tt)* }

        @pending
            { $($found_tag:ident)::+ ($($found_slot_name:ident: $found_slot_typ:ty),+) }
            $($tail:tt)*
    ) => {
        $crate::ocaml_alloc_variant_match!{
            $self, {1u8 + $current_block_tag}, $current_long_tag,

            @units { $($unit_tags_accum)* }
            @blocks {
                $($block_tags_accum)*
                { $($found_tag)::+ ($($found_slot_name: $found_slot_typ),+) @ $current_block_tag }
            }

            @pending $($tail)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ocaml_closure_reference {
    ($var:ident, $name:ident) => {
        static name: &str = stringify!($name);
        static mut OC: Option<$crate::internal::OCamlClosure> = None;
        if OC.is_none() {
            OC = $crate::internal::OCamlClosure::named(name);
        }
        let $var =
            OC.unwrap_or_else(|| panic!("OCaml closure with name '{}' not registered", name));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! gcmark_result {
    ($gc:ident, $obj:expr) => {
        match $obj {
            Ok(t) => Ok(t.mark($gc).eval($gc)),
            Err(e) => Err(e),
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_to_ocaml_unit {
    // No return value, default to unit
    () => ($crate::OCaml<()>);

    // Return value specified
    ($rtyp:ty) => ($rtyp);
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

#[doc(hidden)]
#[macro_export]
macro_rules! expand_args_init {
    ($gc:ident) => ();
    ($gc:ident ,) => ();

    // Nothing is done for unboxed floats
    ($gc:ident, $arg:ident : f64) => ();

    ($gc:ident, $arg:ident : f64, $($args:tt)*) => ($crate::expand_args_init!($gc, $($args)*));

    // Other values are wrapped in `OCaml<T>` as given the same lifetime as the gc handle
    ($gc:ident, $arg:ident : $typ:ty) => (let $arg : $typ = unsafe { $crate::OCaml::new($gc, $arg) };);

    ($gc:ident, $arg:ident : $typ:ty, $($args:tt)*) => {
        let $arg : $typ = unsafe { $crate::OCaml::new($gc, $arg) };
        $crate::expand_args_init!($gc, $($args)*)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function {
    // Final expansions, with all argument types converted

    {
        @name $name:ident
        @gc { $gc:ident $(($($keeper:ident),+))? }
        @final_args { $($arg:ident : $typ:ty,)+ }
        @proc_args { $(,)? }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        #[no_mangle]
        pub extern "C" fn $name( $($arg: $typ),* ) -> $crate::expand_exported_function_return!($($rtyp)*) {
            $crate::ocaml_frame!( $gc $(($($keeper),+))?, {
                $crate::expand_args_init!($gc, $($original_args)*);
                $crate::expand_exported_function_body!(@body $body @return $($rtyp)* )
            })
        }
    };

    // Args processing

    // Next arg is an unboxed float, leave as-is

    {
        @name $name:ident
        @gc { $($gc_decl:tt)+ }
        @final_args { $($final_args:tt)* }
        @proc_args { $next_arg:ident : f64, $($proc_args:tt)* }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!{
            @name $name
            @gc { $($gc_decl)+ }
            @final_args { $($final_args)* $next_arg : f64, }
            @proc_args { $($proc_args)* }
            @return { $($rtyp)* }
            @body $body
            @original_args $($original_args)*
        }
    };

    // Next arg is not an uboxed float, replace with RawOCaml in output

    {
        @name $name:ident
        @gc { $($gc_decl:tt)+ }
        @final_args { $($final_args:tt)* }
        @proc_args { $next_arg:ident : $typ:ty, $($proc_args:tt)* }
        @return { $($rtyp:tt)* }
        @body $body:block
        @original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!{
            @name $name
            @gc { $($gc_decl)+ }
            @final_args { $($final_args)* $next_arg : $crate::RawOCaml, }
            @proc_args { $($proc_args)* }
            @return { $($rtyp)* }
            @body $body
            @original_args $($original_args)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function_body {
    { @body $body:block @return f64 } => {
        #[allow(unused_braces)]
        $body
    };

    { @body $body:block @return $rtyp:ty } => {{
        let retval : $rtyp = $body;
        unsafe { retval.raw() }
    }};

    { @body $body:block @return } => {
        $crate::expand_exported_function_body!(
            @body $body
            @return $crate::OCaml<()>
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function_return {
    () => { $crate::RawOCaml };

    (f64) => { f64 };

    ($rtyp:ty) => { $crate::RawOCaml };
}