// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use ocaml_interop::{
    alloc_error, alloc_ok, BoxRoot, FromOCaml, OCaml, OCamlBytes, OCamlException, OCamlFloat,
    OCamlFloatArray, OCamlInt, OCamlInt32, OCamlInt64, OCamlList, OCamlRuntime, OCamlUniformArray,
    ToOCaml,
};
use std::{thread, time};

#[derive(FromOCaml)]
enum Movement {
    Step {
        #[ocaml(as_ = "OCamlInt")]
        count: i32,
    },
    Expand {
        #[ocaml(as_ = "OCamlInt")]
        width: i64,
        #[ocaml(as_ = "OCamlInt")]
        height: i64,
    },
    RotateLeft,
    RotateRight,
}

#[derive(FromOCaml)]
#[ocaml(polymorphic_variant)]
enum PolymorphicMovement {
    Step {
        #[ocaml(as_ = "OCamlInt")]
        count: i32,
    },
    Expand {
        #[ocaml(as_ = "OCamlInt")]
        width: i64,
        #[ocaml(as_ = "OCamlInt")]
        height: i64,
    },
    RotateLeft,
    RotateRight,
}

#[ocaml_interop::export]
pub fn rust_twice(cr: &mut OCamlRuntime, num: OCaml<OCamlInt>) -> OCaml<OCamlInt> {
    let num: i64 = num.to_rust();
    unsafe { OCaml::of_i64_unchecked(num * 2) }
}

#[ocaml_interop::export]
pub fn rust_twice_boxed_i64(cr: &mut OCamlRuntime, num: OCaml<OCamlInt64>) -> OCaml<OCamlInt64> {
    let num: i64 = num.to_rust();
    let result = num * 2;
    result.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_twice_boxed_i32(cr: &mut OCamlRuntime, num: OCaml<OCamlInt32>) -> OCaml<OCamlInt32> {
    let num: i32 = num.to_rust();
    let result = num * 2;
    result.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_add_unboxed_floats_noalloc(_cr: &mut OCamlRuntime, num: f64, num2: f64) -> f64 {
    num * num2
}

#[ocaml_interop::export]
pub fn rust_twice_boxed_float(cr: &mut OCamlRuntime, num: OCaml<OCamlFloat>) -> OCaml<OCamlFloat> {
    let num: f64 = num.to_rust();
    let result = num * 2.0;
    result.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_twice_unboxed_float(_cr: &mut OCamlRuntime, num: f64) -> f64 {
    num * 2.0
}

#[ocaml_interop::export]
pub fn rust_increment_bytes(
    cr: &mut OCamlRuntime,
    bytes: OCaml<OCamlBytes>,
    first_n: OCaml<OCamlInt>,
) -> OCaml<OCamlBytes> {
    let first_n: i64 = first_n.to_rust();
    let first_n = first_n as usize;
    let mut vec: Vec<u8> = bytes.to_rust();

    for i in 0..first_n {
        vec[i] += 1;
    }

    vec.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_increment_ints_list(
    cr: &mut OCamlRuntime,
    ints: OCaml<OCamlList<OCamlInt>>,
) -> OCaml<OCamlList<OCamlInt>> {
    let mut vec: Vec<i64> = ints.to_rust();

    for i in 0..vec.len() {
        vec[i] += 1;
    }

    vec.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_increment_ints_uniform_array(
    cr: &mut OCamlRuntime,
    ints: OCaml<OCamlUniformArray<OCamlInt>>,
) -> OCaml<OCamlUniformArray<OCamlInt>> {
    let mut vec: Vec<i64> = ints.to_rust();

    for i in 0..vec.len() {
        vec[i] += 1;
    }

    vec.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_increment_floats_uniform_array(
    cr: &mut OCamlRuntime,
    ints: OCaml<OCamlUniformArray<OCamlFloat>>,
) -> OCaml<OCamlUniformArray<OCamlFloat>> {
    let mut vec: Vec<f64> = ints.to_rust();

    for i in 0..vec.len() {
        vec[i] += 1.;
    }

    vec.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_increment_floats_float_array(
    cr: &mut OCamlRuntime,
    ints: OCaml<OCamlFloatArray>,
) -> OCaml<OCamlFloatArray> {
    let mut vec: Vec<f64> = ints.to_rust();

    for i in 0..vec.len() {
        vec[i] += 1.;
    }

    vec.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_make_tuple(
    cr: &mut OCamlRuntime,
    fst: OCaml<String>,
    snd: OCaml<OCamlInt>,
) -> OCaml<(String, OCamlInt)> {
    let fst: String = fst.to_rust();
    let snd: i64 = snd.to_rust();
    let tuple = (fst, snd);
    tuple.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_make_some(cr: &mut OCamlRuntime, value: OCaml<String>) -> OCaml<Option<String>> {
    let value: String = value.to_rust();
    let some_value = Some(value);
    some_value.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_make_ok(
    cr: &mut OCamlRuntime,
    value: OCaml<OCamlInt>,
) -> OCaml<Result<OCamlInt, String>> {
    let value: i64 = value.to_rust();
    let ok_value: Result<i64, String> = Ok(value);
    ok_value.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_make_error(
    cr: &mut OCamlRuntime,
    value: OCaml<String>,
) -> OCaml<Result<OCamlInt, String>> {
    let value: String = value.to_rust();
    let error_value: Result<i64, String> = Err(value);
    error_value.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_sleep_releasing(cr: &mut OCamlRuntime, millis: OCaml<OCamlInt>) {
    let millis: i64 = millis.to_rust();
    cr.releasing_runtime(|| thread::sleep(time::Duration::from_millis(millis as u64)));
}

#[ocaml_interop::export]
pub fn rust_sleep(cr: &mut OCamlRuntime, millis: OCaml<OCamlInt>) {
    let millis: i64 = millis.to_rust();
    thread::sleep(time::Duration::from_millis(millis as u64));
}

#[ocaml_interop::export]
pub fn rust_string_of_movement(cr: &mut OCamlRuntime, movement: OCaml<Movement>) -> OCaml<String> {
    let m = movement.to_rust();
    let s = match m {
        Movement::Step { count } => format!("Step({})", count),
        Movement::Expand { width, height } => {
            format!("Expand {{ width: {width}, height: {height} }}")
        }
        Movement::RotateLeft => "RotateLeft".to_owned(),
        Movement::RotateRight => "RotateRight".to_owned(),
    };
    s.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_string_of_polymorphic_movement(
    cr: &mut OCamlRuntime,
    polymorphic_movement: OCaml<PolymorphicMovement>,
) -> OCaml<String> {
    let pm = polymorphic_movement.to_rust();
    let s = match pm {
        PolymorphicMovement::Step { count } => format!("`Step({})", count),
        PolymorphicMovement::Expand { width, height } => {
            format!("`Expand({}, {})", width, height)
        }
        PolymorphicMovement::RotateLeft => "`RotateLeft".to_owned(),
        PolymorphicMovement::RotateRight => "`RotateRight".to_owned(),
    };
    s.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_call_ocaml_closure(
    cr: &mut OCamlRuntime,
    ocaml_function: BoxRoot<fn(OCamlInt) -> OCamlInt>,
) -> OCaml<Result<OCamlInt, String>> {
    let call_result: Result<i64, String> = ocaml_function
        .try_call(cr, &0i64)
        .map(|call_result| call_result.to_rust())
        .map_err(|exception| exception.message().unwrap_or("no message".to_string()));
    call_result.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_call_ocaml_closure_and_return_exn(
    cr: &mut OCamlRuntime,
    ocaml_function: BoxRoot<fn(OCamlInt) -> OCamlInt>,
) -> OCaml<Result<OCamlInt, OCamlException>> {
    let call_result: Result<OCaml<OCamlInt>, OCaml<OCamlException>> =
        ocaml_function.try_call(cr, &0i64);

    match call_result {
        Ok(value) => {
            let ocaml_value = value.root();
            alloc_ok(cr, &ocaml_value)
        }
        Err(error) => {
            let ocaml_error = error.root();
            alloc_error(cr, &ocaml_error)
        }
    }
}

#[ocaml_interop::export(bytecode = "rust_rust_add_7ints_byte")]
pub fn rust_rust_add_7ints(
    cr: &mut OCamlRuntime,
    int1: OCaml<OCamlInt>,
    int2: OCaml<OCamlInt>,
    int3: OCaml<OCamlInt>,
    int4: OCaml<OCamlInt>,
    int5: OCaml<OCamlInt>,
    int6: OCaml<OCamlInt>,
    int7: OCaml<OCamlInt>,
) -> OCaml<OCamlInt> {
    let int1: i64 = int1.to_rust();
    let int2: i64 = int2.to_rust();
    let int3: i64 = int3.to_rust();
    let int4: i64 = int4.to_rust();
    let int5: i64 = int5.to_rust();
    let int6: i64 = int6.to_rust();
    let int7: i64 = int7.to_rust();
    unsafe { OCaml::of_i64_unchecked(int1 + int2 + int3 + int4 + int5 + int6 + int7) }
}

#[ocaml_interop::export]
pub fn rust_should_panic_with_message(
    cr: &mut OCamlRuntime,
    message: OCaml<String>,
    should_panic: OCaml<bool>,
) {
    let message: String = message.to_rust();
    let should_panic: bool = should_panic.to_rust();
    if should_panic {
        panic!("{message}");
    }
}

#[ocaml_interop::export]
pub fn rust_panic_while_releasing_lock(
    cr: &mut OCamlRuntime,
    message: OCaml<String>,
    should_panic: OCaml<bool>,
) {
    let message_rs: String = message.to_rust();
    let should_panic_rs: bool = should_panic.to_rust();

    cr.releasing_runtime(|| {
        if should_panic_rs {
            // Simulate some work before panicking
            println!("Rust: About to panic while lock is released!");
            panic!("{}", message_rs);
        } else {
            println!("Rust: Executing normally while lock is released.");
        }
    });
}
