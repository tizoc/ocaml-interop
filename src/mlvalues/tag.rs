// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

pub use ocaml_sys::{
    Tag, CLOSURE, DOUBLE_ARRAY, NO_SCAN, STRING, TAG_CONS as CONS, TAG_SOME as SOME,
};

pub const TAG_POLYMORPHIC_VARIANT: Tag = 0;

/// Note that `TAG_EXCEPTION`` is equivalent to `TAG_POLYMORPHIC_VARIANT`, and also
/// corresponds to the tag associated with records and tuples.
pub const TAG_EXCEPTION: Tag = 0;

pub const TAG_OK: Tag = 0;
pub const TAG_ERROR: Tag = 1;
