#[macro_export]
macro_rules! alloc_ocaml {
    { $(($obj:expr).)?$($fn:ident).+($gc:ident) } => {
        {
            let res = $(($obj).)?$($fn).+($crate::GCtoken {});
            res.mark($gc).eval($gc)
        }
    };

    { $(($obj:expr).)?$($fn:ident).+($gc:ident, $($arg:expr),* ) } => {
        {
            let res = $(($obj).)?$($fn).+($crate::GCtoken {}, $($arg),* );
            res.mark($gc).eval($gc)
        }
    };
}

#[macro_export]
macro_rules! call_ocaml {
    { $($fn:ident).+($gc:ident, $arg:expr) } => {
        {
            let res = $($fn).+.call($arg);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr) } => {
        {
            let res = $($fn).+.call2($arg1, $arg2);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $arg1:expr, $arg2:expr, $arg3:expr) } => {
        {
            let res = $($fn).+.call3($arg1, $arg2, $arg3);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };

    { $($fn:ident).+($gc:ident, $($arg:expr),*) } => {
        {
            let mut args = [$($arg.eval()),*];
            let res = $($fn).+.call_n(&mut args);
            res.map(|v| v.mark($gc).eval($gc))
        }
    };
}
