(* This file demonstrates basic tuple interoperability between OCaml and Rust. *)

(* External declaration for the Rust function `process_ocaml_tuple`.
   The type signature `int * string -> string * int` defines how OCaml
   interacts with the Rust function: it sends an (int, string) tuple
   and expects a (string, int) tuple in return.
*)
external process_ocaml_tuple : int * string -> string * int = "process_ocaml_tuple"

let () =
  let my_ocaml_tuple = (42, "hello from OCaml world") in
  Printf.printf "OCaml: Sending tuple to Rust: (%d, \"%s\")\n%!"
    (fst my_ocaml_tuple) (snd my_ocaml_tuple);

  (* Call the Rust function. `ocaml-interop` handles the conversion
     of the OCaml tuple to a type Rust can understand, and vice-versa for the result. *)
  let (returned_string, returned_int) = process_ocaml_tuple my_ocaml_tuple in

  Printf.printf "OCaml: Received tuple from Rust: (\"%s\", %d)\n%!"
    returned_string returned_int;

  (* Expected output from both OCaml and Rust (via Rust's `println!`):
     OCaml: Sending tuple to Rust: (42, "hello from OCaml world")
     [Rust] Individually accessed tuple elements: int = 42, string = "hello from OCaml world"
     [Rust] Converting the full OCaml tuple to a Rust tuple...
     [Rust] Full Rust tuple: (42, "hello from OCaml world")
     [Rust] Processed Rust tuple: (142, "Processed in Rust: hello from OCaml world")
     [Rust] Converting result to OCaml tuple and returning...
     OCaml: Received tuple from Rust: ("Processed in Rust: hello from OCaml world", 142)
  *)
  ()
