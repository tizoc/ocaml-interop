(* Copyright (c) Viable Systems and TezEdge Contributors
   SPDX-License-Identifier: MIT *)

(* Define various OCaml types to test the inspector *)

(* Simple record with different field types *)
type test_record = {
  int_field: int;
  float_field: float;
  string_field: string;
  bool_field: bool;
  tuple_field: int * string * float;
}

(* Variant type *)
type test_variant =
  | Empty
  | WithInt of int
  | WithString of string
  | WithTuple of int * float
  | Complex of test_record

(* List of variants *)
type nested_structure = test_variant list

(* Polymorphic variant *)
type poly_variant = [
  | `None
  | `Int of int
  | `Tuple of int * string
]

(* Function to create test instances *)
let create_test_record () =
  { 
    int_field = 42; 
    float_field = 3.14159;
    string_field = "Hello, OCaml!";
    bool_field = true;
    tuple_field = (123, "tuple element", 45.67)
  }

let create_empty_variant () = Empty

let create_int_variant () = WithInt 42

let create_string_variant () = WithString "variant string"

let create_tuple_variant () = WithTuple (42, 3.14)

let create_complex_variant () = Complex (create_test_record ())

let create_list_of_variants () = [
  Empty;
  WithInt 42;
  WithString "variant in list";
  WithTuple (99, 99.99)
]

let create_poly_none () = `None

let create_poly_int () = `Int 42

let create_poly_tuple () = `Tuple (42, "poly tuple")

(* Register all functions for Rust to call *)
let () =
  Callback.register "create_test_record" create_test_record;
  Callback.register "create_empty_variant" create_empty_variant;
  Callback.register "create_int_variant" create_int_variant;
  Callback.register "create_string_variant" create_string_variant;
  Callback.register "create_tuple_variant" create_tuple_variant;
  Callback.register "create_complex_variant" create_complex_variant;
  Callback.register "create_list_of_variants" create_list_of_variants;
  Callback.register "create_poly_none" create_poly_none;
  Callback.register "create_poly_int" create_poly_int;
  Callback.register "create_poly_tuple" create_poly_tuple
