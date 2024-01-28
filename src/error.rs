// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::{MAX_FIXNUM, MIN_FIXNUM};
use core::fmt;

#[derive(Debug)]
pub enum OCamlFixnumConversionError {
    InputTooBig(i64),
    InputTooSmall(i64),
}

impl fmt::Display for OCamlFixnumConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OCamlFixnumConversionError::InputTooBig(n) => write!(
                f,
                "Input value doesn't fit in OCaml fixnum n={} > MAX_FIXNUM={}",
                n, MAX_FIXNUM
            ),
            OCamlFixnumConversionError::InputTooSmall(n) => write!(
                f,
                "Input value doesn't fit in OCaml fixnum n={} < MIN_FIXNUM={}",
                n, MIN_FIXNUM
            ),
        }
    }
}
