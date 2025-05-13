type action = [
  | `Start
  | `Stop
  | `Set_speed of int
]

external rust_create_action : unit -> action = "rust_create_action"
external rust_process_action : action -> string = "rust_process_action"

let () =
  let created_action = rust_create_action () in
  let desc_created =
    match created_action with
    | `Start -> "`Start"
    | `Stop -> "`Stop"
    | `Set_speed n -> Printf.sprintf "`Set_speed %d" n
  in
  Printf.printf "[OCaml] Action from Rust creation: %s\n" desc_created;

  let processed_desc_created = rust_process_action created_action in
  Printf.printf "[OCaml] Action from Rust (processed): %s\n" processed_desc_created;

  let local_action : action = `Set_speed 75 in
  let processed_desc_local = rust_process_action local_action in
  Printf.printf "[OCaml] Local Action processed by Rust: %s\n" processed_desc_local
