use ocaml_interop::{OCaml, OCamlInt, OCamlRuntime, ToOCaml};

#[ocaml_interop::export]
fn process_ocaml_tuple(
    cr: &mut OCamlRuntime,
    input_tuple: OCaml<(OCamlInt, String)>, // OCaml type: int * string
) -> OCaml<(String, OCamlInt)> {
    // OCaml type: string * int

    // --- Individual Element Access (Demonstration) ---
    // You can access tuple elements individually using .fst() and .snd()
    // This returns OCaml<T> references to the elements.
    let ocaml_int_ref = input_tuple.fst();
    let ocaml_str_ref = input_tuple.snd();

    let individual_rust_int: i64 = ocaml_int_ref.to_rust();
    let individual_rust_string: String = ocaml_str_ref.to_rust();
    println!(
        "[Rust] Individually accessed tuple elements: int = {}, string = \"{}\"",
        individual_rust_int, individual_rust_string
    );

    // --- Full Tuple Conversion ---
    // For processing the entire tuple, convert it directly to a Rust tuple.
    // This is generally more concise if you need all elements.
    println!("[Rust] Converting the full OCaml tuple to a Rust tuple...");
    let (rust_int, rust_string): (i64, String) = input_tuple.to_rust();
    println!(
        "[Rust] Full Rust tuple: ({}, \"{}\")",
        rust_int, rust_string
    );

    let processed_string = format!("Processed in Rust: {}", rust_string);
    let processed_int = rust_int + 100;

    println!(
        "[Rust] Processed Rust tuple: ({}, \"{}\")",
        processed_int, processed_string
    );

    let result_tuple_rust = (processed_string, processed_int);

    println!("[Rust] Converting result to OCaml tuple and returning...");
    result_tuple_rust.to_ocaml(cr)
}
