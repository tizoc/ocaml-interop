(* OCaml variant type definition. *)
(* The order of constructors must match the order in Rust's `impl_conv_ocaml_variant!`. *)
type status =
  | Ok                (* First constructor, no arguments *)
  | Error of string   (* Second constructor, one argument *)
  | Retrying of int   (* Third constructor, one argument *)

(* External declarations for Rust functions. *)
external rust_process_status : status -> string = "rust_process_status"
external rust_create_status_ok : unit -> status = "rust_create_status_ok"
external rust_create_status_error : string -> status = "rust_create_status_error"
external rust_create_status_retrying : int -> status = "rust_create_status_retrying"

(* Helper function to print OCaml status values. *)
let print_status_details (s: status) (source: string) =
  match s with
  | Ok -> Printf.printf "%s: Status is Ok\n%!" source
  | Error msg -> Printf.printf "%s: Status is Error(\"%s\")\n%!" source msg
  | Retrying count -> Printf.printf "%s: Status is Retrying(%d)\n%!" source count

let () =
  Printf.printf "--- Testing OCaml -> Rust status processing ---\n%!";
  let status_ocaml_ok = Ok in
  let status_ocaml_error = Error "An OCaml-side problem" in
  let status_ocaml_retrying = Retrying 3 in

  print_status_details status_ocaml_ok "OCaml (original)";
  let result_ok_processed = rust_process_status status_ocaml_ok in
  Printf.printf "Rust processed to: %s\n%!" result_ok_processed;

  print_status_details status_ocaml_error "OCaml (original)";
  let result_error_processed = rust_process_status status_ocaml_error in
  Printf.printf "Rust processed to: %s\n%!" result_error_processed;

  print_status_details status_ocaml_retrying "OCaml (original)";
  let result_retrying_processed = rust_process_status status_ocaml_retrying in
  Printf.printf "Rust processed to: %s\n%!" result_retrying_processed;

  Printf.printf "--- Testing Rust -> OCaml status creation ---\n%!";

  let status_rust_ok = rust_create_status_ok () in
  print_status_details status_rust_ok "OCaml (from Rust)";

  let status_rust_error = rust_create_status_error "A Rust-side problem" in
  print_status_details status_rust_error "OCaml (from Rust)";

  let status_rust_retrying = rust_create_status_retrying 7 in
  print_status_details status_rust_retrying "OCaml (from Rust)";

  Printf.printf "\nariants example test complete.\n%!"
