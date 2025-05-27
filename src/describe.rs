// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use crate::mlvalues::{
    bigarray::{Array1, BigarrayElt},
    OCamlBytes, OCamlFloat, OCamlInt, OCamlInt32, OCamlInt64, OCamlList,
};
use crate::{OCamlFloatArray, OCamlUniformArray};

pub trait OCamlDescriber {
    fn ocaml_type_name() -> String;
}

impl OCamlDescriber for bool {
    fn ocaml_type_name() -> String {
        "bool".to_string()
    }
}

impl OCamlDescriber for () {
    fn ocaml_type_name() -> String {
        "unit".to_string()
    }
}

impl OCamlDescriber for String {
    fn ocaml_type_name() -> String {
        "string".to_string()
    }
}

// Marker Type Implementations from `ocaml-interop`

impl OCamlDescriber for OCamlInt {
    fn ocaml_type_name() -> String {
        "int".to_string()
    }
}

impl OCamlDescriber for OCamlInt32 {
    fn ocaml_type_name() -> String {
        "int32".to_string()
    }
}

impl OCamlDescriber for OCamlInt64 {
    fn ocaml_type_name() -> String {
        "int64".to_string()
    }
}

impl OCamlDescriber for OCamlFloat {
    fn ocaml_type_name() -> String {
        "float".to_string()
    }
}

impl OCamlDescriber for OCamlBytes {
    fn ocaml_type_name() -> String {
        "bytes".to_string()
    }
}

// Generic marker type implementations

impl<T: OCamlDescriber> OCamlDescriber for Option<T> {
    fn ocaml_type_name() -> String {
        format!("{} option", T::ocaml_type_name())
    }
}

impl<T: OCamlDescriber, E: OCamlDescriber> OCamlDescriber for Result<T, E> {
    fn ocaml_type_name() -> String {
        format!(
            "({}, {}) result",
            T::ocaml_type_name(),
            E::ocaml_type_name()
        )
    }
}

impl<T: OCamlDescriber> OCamlDescriber for OCamlList<T> {
    fn ocaml_type_name() -> String {
        format!("{} list", T::ocaml_type_name())
    }
}

impl<T: OCamlDescriber> OCamlDescriber for OCamlUniformArray<T> {
    fn ocaml_type_name() -> String {
        format!("{} array", T::ocaml_type_name())
    }
}

impl OCamlDescriber for OCamlFloatArray {
    fn ocaml_type_name() -> String {
        "float array".to_string()
    }
}

impl<T: OCamlDescriber + BigarrayElt> OCamlDescriber for Array1<T> {
    fn ocaml_type_name() -> String {
        // This is a simplification. OCaml's Bigarray.Array1.t type is more complex,
        // often showing element kind and layout. e.g. (float, float64_elt, c_layout) Bigarray.Array1.t
        // For now, we'll use the inner type and "bigarray".
        // The actual BigarrayElt might provide more info via another trait later if needed.
        format!("{} bigarray", T::ocaml_type_name())
    }
}
