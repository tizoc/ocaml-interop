#[macro_export]
macro_rules! ocaml_frame {
    ($gc:ident, $body:block) => {{
        let mut frame: $crate::internal::GCFrame = Default::default();
        let $gc = frame.initialize();
        $body
    }};
}

#[macro_export]
macro_rules! ocaml {
    () => ();

    ($vis:vis fn $name:ident($arg:ident: $typ:ty) -> $rtyp:ty; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::internal::GCToken, $arg: $crate::OCaml<$typ>) -> $crate::OCamlResult<$rtyp> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call(token, $arg)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($arg1:ident: $typ1:ty, $arg2:ident: $typ2:ty) -> $rtyp:ty; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::internal::GCToken, $arg1: $crate::OCaml<$typ1>, $arg2: $crate::OCaml<$typ2>) -> $crate::OCamlResult<$rtyp> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call2(token, $arg1, $arg2)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($arg1:ident: $typ1:ty, $arg2:ident: $typ2:ty, $arg3:ident: $typ3:ty) -> $rtyp:ty; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::internal::GCToken, $arg1: $crate::OCaml<$typ1>, $arg2: $crate::OCaml<$typ2>, $arg3: $crate::OCaml<$typ3>) -> $crate::OCamlResult<$rtyp> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call3(token, $arg1, $arg2, $arg3)
        }

        ocaml!($($t)*);
    };

    ($vis:vis fn $name:ident($($arg:ident: $typ:ty),+) -> $rtyp:ty; $($t:tt)*) => {
        $vis unsafe fn $name(token: $crate::internal::GCToken, $($arg: $crate::OCaml<$typ>),+) -> $crate::OCamlResult<$rtyp> {
            $crate::ocaml_closure_reference!(F, $name);
            F.call_n(token, &mut [$($arg.raw()),+])
        }

        ocaml!($($t)*);
    }
}

#[macro_export]
macro_rules! ocaml_export {
    {
      $(
        fn $name:ident( $gc:ident, $($arg:ident : $ty:ty),* ) -> $res:ty
           $body:block
      )*
    } => {
      $(
        #[no_mangle]
        pub extern "C" fn $name( $($arg: $crate::RawOCaml), *) -> $crate::RawOCaml {
            $crate::ocaml_frame!($gc, {
                $(let $arg : $ty = unsafe { OCaml::new($gc, $arg) };)*
                let retval : $res = $body;
                unsafe { retval.raw() }
            })
        }
      )*
    }
}

#[macro_export]
macro_rules! ocaml_alloc {
    ( $(($obj:expr).)?$($fn:ident).+($gc:ident) ) => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() });
            res.mark($gc).eval($gc)
        }
    };

    ( $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),* ) ) => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() }, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };

    ( $obj:literal.$($fn:ident).+($gc:ident) ) => {
        {
            let res = $obj.$($fn).+(unsafe { $gc.token() });
            res.mark($gc).eval($gc)
        }
    };

    ( $obj:literal.$($fn:ident).+($gc:ident, $($arg:expr),* ) ) => {
        {
            let res = $obj.$($fn).+(unsafe { $gc.token() }, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };
}

#[macro_export]
macro_rules! ocaml_call {
    ( $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),* ) ) => {
        {
            let res = unsafe { $(($obj).)?$($fn).+($gc.token(), $($arg),* ) };
            $crate::gcmark_result!($gc, res)
        }
    };

    ( $($path:ident)::+($gc:ident, $($args:expr),+) ) => {
        {
            let res = unsafe { $($path)::+($gc.token(), $($args),+) };
            $crate::gcmark_result!($gc, res)
        }
    };

    ( $($path:ident)::+.$($field:ident).+($gc:ident, $($args:expr),+) ) => {
        {
            let res = unsafe { $($path)::+$($field).+($gc.token(), $($args),+) };
            $crate::gcmark_result!($gc, res)
        }
    };
}

// Utility macros

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
