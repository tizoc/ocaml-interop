// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

/// `ocaml_frame!(gc, { ... })` opens a new frame, with the GC handle bound to `gc`. Code inside the passed block
/// can allocate OCaml values and call OCaml functions.
///
/// The variant `ocaml_frame!(gc nokeep, { ... })` (with `nokeep` after the GC handle variable name) avoids
/// setting up the frame and can be used for blocks that do not keep OCaml pointers across OCaml calls
/// (`gc.keep(value)` cannot be used). Keep in mind that this variant is likely to change in the future.
// ocaml_frame!(gc, { ... })
#[macro_export]
macro_rules! ocaml_frame {
    ($gc:ident nokeep, $body:block) => {{
        let mut frame: $crate::internal::GCFrameNoKeep = Default::default();
        let $gc = frame.initialize();
        $body
    }};

    ($gc:ident, $body:block) => {{
        let mut frame: $crate::internal::GCFrame = Default::default();
        let $gc = frame.initialize();
        $body
    }};
}

/// `ocaml! { pub fn ocaml_name(arg1: typ1, ...) -> ret_typ; ... }` declares a function that has been
/// defined in OCaml code and registered with `Callback.register "ocaml_name" the_function`.
/// Visibility and return value type can be omitted. If the return type is omitted, it defaults to
/// unit.
/// `ocaml! { pub alloc fn alloc_name(arg1: typ1, ...) -> ret_typ; ... }` (with `alloc` annotation) defines
/// a record allocation function. In this case, an OCaml counterpart registered with `Callback.register "alloc_name" the_function`
/// is not required (and will not be used if present).
#[macro_export]
macro_rules! ocaml {
    () => ();

    ($vis:vis alloc fn $name:ident($($field:ident: $typ:ty),+ $(,)?) -> $rtyp:ty; $($t:tt)*) => {
        $vis unsafe fn $name(_token: $crate::OCamlAllocToken, $($field: &dyn $crate::ToOCaml<$typ>),+) -> $crate::OCamlAllocResult<$rtyp> {
            $crate::ocaml_frame!(gc, {
                let mut current = 0;
                let mut field_count = $crate::count_fields!($($field)*);
                let record = gc.keep_raw($crate::internal::caml_alloc(field_count, 0));
                $(
                    let $field: $crate::OCaml<$typ> = $crate::to_ocaml!(gc, $field);
                    $crate::internal::store_field(record.get_raw(), current, $field.raw());
                    current += 1;
                )+
                $crate::OCamlAllocResult::of(record.get_raw())
            })
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($arg:ident: $typ:ty $(,)?) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::OCamlAllocToken, $arg: $crate::OCaml<$typ>) -> $crate::OCamlResult<$crate::default_to_unit!($(-> $rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call(token, $arg)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($arg1:ident: $typ1:ty, $arg2:ident: $typ2:ty $(,)?) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::OCamlAllocToken, $arg1: $crate::OCaml<$typ1>, $arg2: $crate::OCaml<$typ2>) -> $crate::OCamlResult<$crate::default_to_unit!($(-> $rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call2(token, $arg1, $arg2)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($arg1:ident: $typ1:ty, $arg2:ident: $typ2:ty, $arg3:ident: $typ3:ty $(,)?) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::OCamlAllocToken, $arg1: $crate::OCaml<$typ1>, $arg2: $crate::OCaml<$typ2>, $arg3: $crate::OCaml<$typ3>) -> $crate::OCamlResult<$crate::default_to_unit!($(-> $rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call3(token, $arg1, $arg2, $arg3)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($($arg:ident: $typ:ty),+ $(,)?) $(-> $rtyp:ty)?; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::OCamlAllocToken, $($arg: $crate::OCaml<$typ>),+) -> $crate::OCamlResult<$crate::default_to_unit!($(-> $rtyp)?)> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call_n(token, &mut [$($arg.raw()),+])
        }

        ocaml!($($t)*);
    }
}

// ocaml_export! { fn export_name(gc, arg1: typ1, ...) -> res_typ ... }
// ocaml_export! { fn export_name(gc, arg1: typ1, ...) ... }
// If no return type is provided, defaults to unit
#[macro_export]
macro_rules! ocaml_export {
    {} => ();

    // Unboxed float return
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($args:tt)*) -> f64
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, @proc_args $($args)*) -> f64
                $body
            #original_args $($args)*
        );

        ocaml_export!{$($t)*}
    };

    // Other return values
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($args:tt)*) $(-> $rtyp:ty)?
           $body:block

        $($t:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, @proc_args $($args)*) $(-> $rtyp)?
                $body
            #original_args $($args)*
        );

        ocaml_export!{$($t)*}
    };
}

// ocaml_alloc!(expr.to_ocaml(gc, ...)))
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

#[macro_export]
macro_rules! to_ocaml {
    ($gc:ident, $obj:expr) => {
        $crate::ocaml_alloc!(($obj).to_ocaml($gc))
    }
}

// ocaml_call!(expr.func(gc, arg1, ...))
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

#[macro_export]
macro_rules! ocaml_unpack_record {
    ($var:ident => $cons:ident {
        $($field:ident : $ocaml_typ:ty),+ $(,)?
    }) => {
        let record = $var;
        unsafe {
            let mut current = 0;

            $(
                current += 1;
                let $field = record.field::<$ocaml_typ>(current - 1).into_rust();
            )+

            $cons {
                $($field),+
            }
        }
    };
}

#[macro_export]
macro_rules! ocaml_alloc_record {
    ($self:ident => $cons:ident {
        $($field:ident : $ocaml_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe {
            $crate::ocaml_frame!(gc, {
                let mut current = 0;
                let field_count = $crate::count_fields!($($field)*);
                let record = gc.keep_raw($crate::internal::caml_alloc(field_count, 0));
                $(
                    let ref $field = $crate::prepare_field_for_mapping!($self.$field $(=> $conv_expr)?);
                    let $field: $crate::OCaml<$ocaml_typ> = $crate::to_ocaml!(gc, $field);
                    current += 1;
                    $crate::internal::store_field(record.get_raw(), current - 1, $field.raw());
                )+
                $crate::OCamlAllocResult::of(record.get_raw())
            })
        }
    };
}

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


#[macro_export]
macro_rules! impl_to_ocaml_record {
    ($rust_typ:ty => $ocaml_typ:ident {
        $($field:ident : $ocaml_field_typ:ty $(=> $conv_expr:expr)?),+ $(,)?
    }) => {
        unsafe impl $crate::ToOCaml<$ocaml_typ> for $rust_typ {
            fn to_ocaml(&self, _token: $crate::OCamlAllocToken) -> $crate::OCamlAllocResult<$ocaml_typ> {
                $crate::ocaml_alloc_record! {
                    self => $ocaml_typ {
                        $($field : $ocaml_field_typ $(=> $conv_expr)?),+
                    }
                }
            }
        }
    };

    ($both_typ:ident {
        $($t:tt)*
    }) => {
        impl_to_ocaml_record! {
            $both_typ => $both_typ {
                $($t)*
            }
        }
    };
}

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

// Utility macros

#[doc(hidden)]
#[macro_export]
macro_rules! count_fields {
    () => {0usize};
    ($_f1:ident $_f2:ident $_f3:ident $_f4:ident $_f5:ident $($fields:ident)*) => {5usize + $crate::count_fields!($($fields)*)};
    ($field:ident $($fields:ident)*) => {1usize + $crate::count_fields!($($fields)*)};
}

#[doc(hidden)]
#[macro_export]
macro_rules! prepare_field_for_mapping {
    ($self:ident.$field:ident) => {
        $self.$field
    };

    ($self:ident.$field:ident => $conv_expr:expr) => {
        {
            let ref $field = $self.$field;
            $conv_expr
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! unpack_variant_tag {
    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+) => {
        $crate::unpack_variant_tag!($self, $current_block_tag, $current_long_tag, $($tag)::+ => $($tag)::+)
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+ => $conv:expr) => {
        $current_long_tag += 1;
        if $self.is_long() && unsafe { $self.raw() } == $current_long_tag - 1 {
            return Ok($conv);
        }
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+)) => {
        $crate::unpack_variant_tag!($self, $current_block_tag, $current_long_tag, $($tag)::+ ($($slot_name: $slot_typ),+) => $($tag)::+($($slot_name),+))
    };

    ($self:ident, $current_block_tag:ident, $current_long_tag:ident, $($tag:ident)::+ ($($slot_name:ident: $slot_typ:ty),+) => $conv:expr) => {
        $current_block_tag += 1;
        if $self.is_block() && $self.tag_value() == $current_block_tag - 1 {
            let mut current_field = 0;

            $(
                current_field += 1;
                let $slot_name = unsafe { $self.field::<$slot_typ>(current_field - 1).into_rust() };
            )+

            return Ok($conv);
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
    (-> $rtyp:ty) => ($rtyp);
}

#[doc(hidden)]
#[macro_export]
macro_rules! default_to_unit {
    // No return value, default to unit
    () => {
        ()
    };

    // Return value specified
    (-> $rtyp:ty) => {
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
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        #[no_mangle]
        pub extern "C" fn $name( $($arg: $typ),* ) -> $crate::RawOCaml {
            $crate::ocaml_frame!( $gc $($nokeep)?, {
                $crate::expand_args_init!($gc, $($original_args)*);
                let retval : $crate::default_to_ocaml_unit!($(-> $rtyp)?) = $body;
                unsafe { retval.raw() }
            })
        }
    };

    // Handle unboxed floats

    // fn func(gc, @proc_args arg: f64)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : f64) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $next_arg : f64, @proc_args) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, @proc_args arg: f64, ...)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : f64, $($proc_args:tt)*) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $next_arg : f64, @proc_args $($proc_args)*) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: f64)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : f64) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : f64, @proc_args) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: f64, ....)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : f64, $($proc_args:tt)*) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : f64, @proc_args $($proc_args)*) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // Handle other types

    // fn func(gc, @proc_args arg: typ)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : $next_typ:ty) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $next_arg : $crate::RawOCaml, @proc_args) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, @proc_args arg: typ, ...)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : $next_typ:ty, $($proc_args:tt)*) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $next_arg : $crate::RawOCaml, @proc_args $($proc_args)*) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: typ)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : $next_typ:ty) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : $crate::RawOCaml, @proc_args) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: typ, ....)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : $next_typ:ty, $($proc_args:tt)*) $(-> $rtyp:ty)?
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : $crate::RawOCaml, @proc_args $($proc_args)*) $(-> $rtyp)?
                $body
            #original_args $($original_args)*
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! expand_exported_function_with_unboxed_float_return {
    // Final expansions, with all argument types converted
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        #[no_mangle]
        pub extern "C" fn $name( $($arg: $typ),* ) -> f64 {
            $crate::ocaml_frame!( $gc $($nokeep)?, {
                $crate::expand_args_init!($gc, $($original_args)*);
                $body
            })
        }
    };

    // Handle unboxed floats

    // fn func(gc, @proc_args arg: f64)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : f64) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $next_arg : f64, @proc_args) -> f64              $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, @proc_args arg: f64, ...)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : f64, $($proc_args:tt)*) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $next_arg : f64, @proc_args $($proc_args)*) -> f64              $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: f64)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : f64) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : f64, @proc_args) -> f64
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: f64, ....)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : f64, $($proc_args:tt)*) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : f64, @proc_args $($proc_args)*) -> f64
                $body
            #original_args $($original_args)*
        );
    };

    // Handle other types

    // fn func(gc, @proc_args arg: typ)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : $next_typ:ty) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $next_arg : $next_typ, @proc_args) -> f64
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, @proc_args arg: typ, ...)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, @proc_args $next_arg:ident : $next_typ:ty, $($proc_args:tt)*) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $next_arg : $next_typ, @proc_args $($proc_args)*) -> f64
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: typ)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : $next_typ:ty) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : $next_typ, @proc_args) -> f64
                $body
            #original_args $($original_args)*
        );
    };

    // fn func(gc, arg1: typ1, ..., @proc_args arg: typ, ....)
    {
        fn $name:ident( $gc:ident $($nokeep:ident)?, $($arg:ident : $typ:ty),+, @proc_args $next_arg:ident : $next_typ:ty, $($proc_args:tt)*) -> f64
           $body:block
        #original_args $($original_args:tt)*
    } => {
        $crate::expand_exported_function_with_unboxed_float_return!(
            fn $name( $gc $($nokeep)?, $($arg : $typ),*, $next_arg : $next_typ, @proc_args $($proc_args)*) -> f64
                $body
            #original_args $($original_args)*
        );
    };
}
