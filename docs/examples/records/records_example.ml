(* Define the OCaml record type. Field order must match `impl_conv_ocaml_record!`. *)
type person = {
  full_name : string;
  birth_year : int;
  is_active : bool;
}

(* Declare the external Rust functions *)
external rust_update_person_activity : person -> bool -> person = "rust_update_person_activity"
external rust_create_person : string -> int -> bool -> person = "rust_create_person"

let print_person_status p =
  Printf.printf "OCaml: Person Name: %s, Birth Year: %d, Active: %b\n%!"
    p.full_name p.birth_year p.is_active

let () =
  let initial_person = { full_name = "John Doe"; birth_year = 1990; is_active = true } in
  Printf.printf "Initial person:\n%!";
  print_person_status initial_person;

  let updated_person = rust_update_person_activity initial_person false in
  Printf.printf "\nUpdated person (activity changed to false by Rust):\n%!";
  print_person_status updated_person;

  let new_person_from_rust = rust_create_person "Alice Wonderland" 2000 true in
  Printf.printf "\nNew person created by Rust:\n%!";
  print_person_status new_person_from_rust;

  Printf.printf "\nTest complete.\n%!"
