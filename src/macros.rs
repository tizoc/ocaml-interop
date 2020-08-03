#[macro_export]
macro_rules! ocaml_frame {
    ($gc:ident, $body:block) => {
        {
            let mut frame: $crate::GCFrame = Default::default();
            let $gc = frame.initialize();
            $body
        }
    };
}

#[macro_export]
macro_rules! ocaml_alloc {
    { $(($obj:expr).)?$($fn:ident).+($gc:ident) } => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() });
            res.mark($gc).eval($gc)
        }
    };

    { $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),* ) } => {
        {
            let res = $(($obj).)?$($fn).+(unsafe { $gc.token() }, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };
}

#[macro_export]
macro_rules! ocaml_call {

    // Field access: value.field.access(gc, ...)

    { $($fn:ident).+($gc:ident, $arg:expr) } => {
        {
            let res = $($fn).+.call(unsafe { $gc.token() }, $arg);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr) } => {
        {
            let res = $($fn).+.call2(unsafe { $gc.token() }, $arg1, $arg2);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr, $arg3:expr) } => {
        {
            let res = $($fn).+.call3(unsafe { $gc.token() }, $arg1, $arg2, $arg3);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $($arg:expr),*) } => {
        {
            let mut args = [$($arg.eval()),*];
            let res = $($fn).+.call_n(unsafe { $gc.token() }, &mut args);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    // Path: path::to::value(gc, ...)

    { $($path:ident)::+($gc:ident, $arg:expr) } => {
        {
            let res = $($path)::+.call(unsafe { $gc.token() }, $arg);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+($gc:ident, $arg1:expr, $arg2:expr) } => {
        {
            let res = $($path)::+.call2(unsafe { $gc.token() }, $arg1, $arg2);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+($gc:ident, $arg1:expr, $arg2:expr, $arg3:expr) } => {
        {
            let res = $($path)::+.call3(unsafe { $gc.token() }, $arg1, $arg2, $arg3);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+($gc:ident, $($arg:expr),*) } => {
        {
            let mut args = [$($arg.eval()),*];
            let res = $($path)::+.call_n(unsafe { $gc.token() }, &mut args);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    // Path + field access: path::to::value.field.access(gc, ...)

    { $($path:ident)::+.$($fn:ident).+($gc:ident, $arg:expr) } => {
        {
            let res = $($path)::+.$($fn).+.call(unsafe { $gc.token() }, $arg);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+.$($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr) } => {
        {
            let res = $($path)::+.$($fn).+.call2(unsafe { $gc.token() }, $arg1, $arg2);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+.$($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr, $arg3:expr) } => {
        {
            let res = $($path)::+.$($fn).+.call3(unsafe { $gc.token() }, $arg1, $arg2, $arg3);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($path:ident)::+.$($fn:ident).+($gc:ident, $($arg:expr),*) } => {
        {
            let mut args = [$($arg.eval()),*];
            let res = $($path)::+.$($fn).+.call_n(unsafe { $gc.token() }, &mut args);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };
}
