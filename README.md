# ocaml-interop

![build](https://github.com/simplestaking/ocaml-interop/workflows/build/badge.svg)
[![crate](https://img.shields.io/crates/v/ocaml-interop.svg)](https://crates.io/crates/ocaml-interop)
[![documentation](https://docs.rs/ocaml-interop/badge.svg)](https://docs.rs/ocaml-interop)
[![license](https://img.shields.io/crates/l/ocaml-interop.svg)](https://github.com/simplestaking/ocaml-interop/blob/master/LICENSE)

_Zinc-iron alloy coating is used in parts that need very good corrosion protection._

**API IS CONSIDERED UNSTABLE AT THE MOMENT AND IS LIKELY TO CHANGE IN THE FUTURE**

[ocaml-interop](https://github.com/simplestaking/ocaml-interop) is an OCaml<->Rust FFI with an emphasis on safety inspired by [caml-oxide](https://github.com/stedolan/caml-oxide) and [ocaml-rs](https://github.com/zshipko/ocaml-rs).

Read the full documentation [here](https://docs.rs/ocaml-interop/).

Report issues on [Github](https://github.com/simplestaking/ocaml-interop/issues).

## Table of Contents

- [How does it work](#how-does-it-work)
- [A quick taste](#a-quick-taste)
- [References and links](#references-and-links)

## How does it work

ocaml-interop, just like [caml-oxide](https://github.com/stedolan/caml-oxide), encodes the invariants of OCaml's garbage collector into the rules of Rust's borrow checker. Any violation of these invariants results in a compilation error produced by Rust's borrow checker.

## A quick taste

### Convert between plain OCaml and Rust values

```rust
let rust_string = ocaml_string.to_rust();
// `cr` = OCaml runtime handle
let new_ocaml_string = to_ocaml!(cr, rust_string);
```

### Convert between Rust and OCaml structs/records

```ocaml
(* OCaml *)
type my_record = {
  string_field: string;
  tuple_field: (string * int);
}
```

```rust
// Rust
struct MyStruct {
    string_field: String,
    tuple_field: (String, i64),
}

impl_conv_ocaml_record! {
    MyStruct {
        string_field: String,
        tuple_field: (String, i64),
    }
}

// ...

let rust_struct = ocaml_record.to_rust();
let new_ocaml_record = to_ocaml!(cr, rust_struct);
```

### Convert between OCaml and Rust variants/enums

```ocaml
(* OCaml *)
type my_variant =
  | EmptyTag
  | TagWithInt of int
```

```rust
// Rust
enum MyEnum {
    EmptyTag,
    TagWithInt(i64),
}

impl_conv_ocaml_variant! {
    MyEnum {
        EmptyTag,
        TagWithInt(OCamlInt),
    }
}

// ...

let rust_enum = ocaml_variant.to_rust();
let new_ocaml_variant = to_ocaml!(cr, rust_enum);
```

### Call OCaml functions from Rust

```ocaml
(* OCaml *)
Callback.register "ocaml_print_endline" print_endline
```

```rust
ocaml! {
    fn ocaml_print_endline(s: String);
}

// ...

let ocaml_string = to_ocaml!(cr, "hello OCaml!", root_var);
ocaml_print_endline(cr, ocaml_string);
```

### Call Rust functions from OCaml

```rust
// Rust
ocaml_export! {
    pub fn twice_boxed_int(cr, num: OCaml<OCamlInt64>) -> OCaml<OCamlInt64> {
        let num = num.to_rust();
        let result = num * 2;
        to_ocaml!(cf, result)
    }
}
```

```ocaml
(* OCaml *)
external rust_twice_boxed_int: int64 -> int64 = "twice_boxed_int"

(* ... *)

let result = rust_twice_boxed_int 123L in
(* ... *)
```

## References and links

- OCaml Manual: [Chapter 20  Interfacing C with OCaml](https://caml.inria.fr/pub/docs/manual-ocaml/intfc.html).
- [Safely Mixing OCaml and Rust](https://docs.google.com/viewer?a=v&pid=sites&srcid=ZGVmYXVsdGRvbWFpbnxtbHdvcmtzaG9wcGV8Z3g6NDNmNDlmNTcxMDk1YTRmNg) paper by Stephen Dolan.
- [Safely Mixing OCaml and Rust](https://www.youtube.com/watch?v=UXfcENNM_ts) talk by Stephen Dolan.
- [CAMLroot: revisiting the OCaml FFI](https://arxiv.org/abs/1812.04905).
- [caml-oxide](https://github.com/stedolan/caml-oxide), the code from that paper.
- [ocaml-rs](https://github.com/zshipko/ocaml-rs), another OCaml<->Rust FFI library.
