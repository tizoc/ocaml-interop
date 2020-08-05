# ZnFe

_Zinc-iron alloy coating is used in parts that need very good corrosion protection._

OCaml<->Rust FFI with an emphasis on safety based on the ideas of [caml-oxide](https://github.com/stedolan/caml-oxide).

Status: **UNSTABLE**

## Usage

### Rules

There are a few rules that have to be followed when calling into the OCaml runtime:

**Rule 1**: Calls into the OCaml runtime that perform allocations should only occur inside `ocaml_frame!` blocks. This applies both to declared OCaml functions and conversions from Rust values into OCaml values.

**TODO**: good/bad example.

**Rule 2**: OCaml values that are obtained as a result of calling an OCaml function can only be referenced directly until another call to an OCaml function happens. This is enforced by Rust's borrow checker.

**TODO**: good/bad example.

**Rule 3**: OCaml values that are the result of an allocation by the OCaml runtime cannot escape the `ocaml_frame!` block inside which they where created. This is enforced by Rust's borrow checker.

**TODO**: good/bad example.

### Calling into OCaml

```rust
use znfe::{
    ocaml_alloc, ocaml_call, ocaml_frame, FromOCaml, OCaml, OCamlRef, ToOCaml, ToOCamlInteger,
};

// To call an OCaml function, it first has to be declared inside an `ocaml!` macro block:
mod ocaml_funcs {
    use znfe::{ocaml, Intnat};

    ocaml! {
        // OCaml: `val increment_bytes: bytes -> int -> bytes`
        // registered with `Callback.register "increment_bytes" increment_bytes`
        pub fn increment_bytes(bytes: String, first_n: Intnat) -> String;
        // OCaml: `val twice: int -> int`
        // registered with `Callback.register "twice" twice`
        pub fn twice(num: Intnat) -> Intnat;
    }

    // The two OCaml functions declared above can now be invoked with the
    // `ocaml_call!` macro: `ocaml_call!(func_name(gc, args...))`.
    // Note the first `gc` parameter, it is an OCaml Garbage Collector handle, and
    // it is obtained by opening a new GC frame block, sung the `ocaml_frame!` macro.
}

fn increment_bytes(bytes1: String, bytes2: String, first_n: usize) -> (String, String) {
    // Any calls into the OCaml runtime have to happen inside an
    // `ocaml_frame!` block. Inside this block, OCaml allocations and references
    // to OCaml allocated values are tracked and validated by Rust's borrow checker.
    // The first argument to the macro is a name for the GC handle, the second
    // is the block of code that will run inside that frame.
    ocaml_frame!(gc, {
        // The `ToOCaml` trait provides the `to_ocaml` function to convert Rust
        // values into OCaml values. Because such conversions usually require
        // the OCaml runtime to perform an allocation, calls to `to_ocaml` have
        // to be made by using the `ocaml_alloc!` macro, and a GC handle has
        // to be passed as an argument.
        let ocaml_bytes1: OCaml<String> = ocaml_alloc!(bytes1.to_ocaml(gc));

        // `ocaml_bytes1` is going to be referenced later, but there calls into the
        // OCaml runtime that perform allocations happening before this value is used again.
        // Those calls into the OCaml runtime invalidate this reference, so it has to be
        // kept alive somehow. To do so, `gc.keep(ocaml_bytes1)` is used. It returns
        // a reference to an OCaml value that is going to be valid during the scope of
        // the current `ocaml_frame!` block. Later `gc.get(the_reference)` can be used
        // to obtain the kept value.
        let ref bytes1_ref: OCamlRef<String> = gc.keep(ocaml_bytes1);

        // Same as above. Note that if we waited to perform this conversion
        // until after `ocaml_bytes1` is used, no references would have to be
        // kept for either of the two OCaml values, because they would be
        // used immediately, with no allocations being performed by the
        // OCaml runtime in-between.
        let ocaml_bytes2: OCaml<String> = ocaml_alloc!(bytes2.to_ocaml(gc));
        let ref bytes2_ref: OCamlRef<String> = gc.keep(ocaml_bytes2);

        // Rust numbers can be converted into OCaml fixnums with the `ToOCamlInteger`
        // trait. Such conversion doesn't require any allocation on the OCaml side,
        // so this call doesn't have to be wrapped by `ocaml_alloc!`, and no GC handle
        // is passed as an argument.
        let ocaml_first_n = (first_n as i64).to_ocaml_fixnum();

        // To call an OCaml function (declared above in a `ocaml!` block) the
        // `ocaml_call!` macro is used. The GC handle has to be passed as the first argument,
        // before all the other declared arguments.
        // The result of this call is a Result<OCamlValue<T>, znfe::Error>, with `Err(...)`
        // being the result of calls for which the OCaml runtime raises an exception.
        let result1 = ocaml_call!(ocaml_funcs::increment_bytes(
            gc,
            // The reference created above is used here to obtain the value
            // of `ocaml_bytes1`
            gc.get(bytes1_ref),
            ocaml_first_n
        ))
        .unwrap();

        // Perform the conversion of the OCaml result value into a
        // Rust value while the reference is still valid because the
        // `ocaml_call!` that follows will invalidate it.
        // Alternatively, the result of `gc.keep(result1)` could be used
        // to be able to reference the value later through an `OCamlRef` value.
        let new_bytes1 = String::from_ocaml(result1);
        let result2 = ocaml_call!(ocaml_funcs::increment_bytes(
            gc,
            gc.get(bytes2_ref),
            ocaml_first_n
        ))
        .unwrap();

        // The `FromOCaml` trait provides the `from_ocaml` function to convert from
        // OCaml values into OCaml values. Unlike the `to_ocaml` function, it doesn't
        // require a GC handle argument, because no allocation is performed by the
        // OCaml runtime when converting into Rust values.
        (new_bytes1, String::from_ocaml(result2))
    })
}

fn twice(num: usize) -> usize {
    ocaml_frame!(gc, {
        let ocaml_num = (num as i64).to_ocaml_fixnum();
        let result = ocaml_call!(ocaml_funcs::twice(gc, ocaml_num));
        i64::from_ocaml(result.unwrap()) as usize
    })
}

fn main() {
    let first_n = twice(5);
    let bytes1 = "000000000000000".to_owned();
    let bytes2 = "aaaaaaaaaaaaaaa".to_owned();
    println!("Bytes1 before: {}", bytes1);
    println!("Bytes2 before: {}", bytes2);
    let (result1, result2) = increment_bytes(bytes1, bytes2, first_n);
    println!("Bytes1 after: {}", result1);
    println!("Bytes2 after: {}", result2);
}
```

## References and links

- OCaml Manual: [Chapter 20  Interfacing C with OCaml](https://caml.inria.fr/pub/docs/manual-ocaml/intfc.html).
- [Safely Mixing OCaml and Rust](https://docs.google.com/viewer?a=v&pid=sites&srcid=ZGVmYXVsdGRvbWFpbnxtbHdvcmtzaG9wcGV8Z3g6NDNmNDlmNTcxMDk1YTRmNg) paper by Stephen Dolan.
- [Safely Mixing OCaml and Rust](https://www.youtube.com/watch?v=UXfcENNM_ts) talk by Stephen Dolan.
- [caml-oxide](https://github.com/stedolan/caml-oxide), the code from that paper.
- [ocaml-rs](https://github.com/zshipko/ocaml-rs), another OCaml<->Rust FFI library.
