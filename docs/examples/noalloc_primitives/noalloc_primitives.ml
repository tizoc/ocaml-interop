(* OCaml-calls-Rust: noalloc_primitives example *)

(* External declarations for Rust functions exported with noalloc *)
external rust_process_float : float -> float = "" "rust_process_float" [@@noalloc] [@@unboxed]
external rust_process_int64 : int64 -> int64 = "" "rust_process_int64" [@@noalloc] [@@unboxed]
external rust_process_bool : (bool [@untagged]) -> (bool [@untagged]) = "" "rust_process_bool" [@@noalloc]
external rust_process_unit_to_unit : (unit [@untagged]) -> (unit [@untagged]) = "" "rust_process_unit_to_unit" [@@noalloc]
external rust_process_isize : (int [@untagged]) -> (int [@untagged]) = "" "rust_process_isize" [@@noalloc]
external rust_process_i32 : int32 -> int32 = "" "rust_process_i32" [@@noalloc] [@@unboxed]

external rust_combine_primitives_noalloc : (float [@unboxed]) -> (int64 [@unboxed]) -> (bool [@untagged]) -> (int64 [@unboxed]) = "" "rust_combine_primitives_noalloc" [@@noalloc]
external rust_select_i64_noalloc : (float [@unboxed]) -> (int64 [@unboxed]) -> (bool [@untagged]) -> (int64 [@unboxed]) = "" "rust_select_i64_noalloc" [@@noalloc]
external rust_log_primitives_noalloc : (float [@unboxed]) -> (int64 [@unboxed]) -> (bool [@untagged]) -> unit = "" "rust_log_primitives_noalloc" [@@noalloc]

let () =
  print_endline "--- OCaml calling Rust (noalloc examples) ---"; flush stdout;

  (* Test rust_process_float *)
  let float_val = 3.14159 in
  let float_res = rust_process_float float_val in
  Printf.printf "OCaml: rust_process_float(%f) = %f\n%!" float_val float_res;
  assert (float_res = float_val *. 2.0);

  (* Test rust_process_int64 *)
  let i64_val = 1234567890123456789L in
  let i64_res = rust_process_int64 i64_val in
  Printf.printf "OCaml: rust_process_int64(%Ld) = %Ld\n%!" i64_val i64_res;
  assert (i64_res = Int64.add i64_val 100L);

  (* Test rust_process_bool *)
  let bool_val_true = true in
  let bool_res_false = rust_process_bool bool_val_true in
  Printf.printf "OCaml: rust_process_bool(%b) = %b\n%!" bool_val_true bool_res_false;
  assert (bool_res_false = not bool_val_true);
  let bool_val_false = false in
  let bool_res_true = rust_process_bool bool_val_false in
  Printf.printf "OCaml: rust_process_bool(%b) = %b\n%!" bool_val_false bool_res_true;
  assert (bool_res_true = not bool_val_false);

  (* Test rust_process_unit_to_unit *)
  print_endline "OCaml: calling rust_process_unit_to_unit..."; flush stdout;
  rust_process_unit_to_unit (); (* Implicitly asserts no exception *)
  print_endline "OCaml: rust_process_unit_to_unit returned."; flush stdout;

  (* Test rust_process_isize *)
  let isize_val = 42 in
  let isize_res = rust_process_isize isize_val in
  Printf.printf "OCaml: rust_process_isize(%d) = %d\n%!" isize_val isize_res;
  assert (isize_res = isize_val + 5);
  let isize_val_neg = -10 in
  let isize_res_neg = rust_process_isize isize_val_neg in
  Printf.printf "OCaml: rust_process_isize(%d) = %d\n%!" isize_val_neg isize_res_neg;
  assert (isize_res_neg = isize_val_neg + 5);

  (* Test rust_process_i32 *)
  let i32_val = 1073741823l in (* Max int32 / 2 roughly *)
  let i32_res = rust_process_i32 i32_val in
  Printf.printf "OCaml: rust_process_i32(%ldl) = %ldl\n%!" i32_val i32_res;
  assert (i32_res = Int32.mul i32_val 2l);
  let i32_val_neg = -536870912l in
  let i32_res_neg = rust_process_i32 i32_val_neg in
  Printf.printf "OCaml: rust_process_i32(%ldl) = %ldl\n%!" i32_val_neg i32_res_neg;
  assert (i32_res_neg = Int32.mul i32_val_neg 2l);

  (* Test rust_combine_primitives_noalloc *)
  let f_comb = 10.5 in
  let i_comb = 20L in
  let b_comb_true = true in
  let b_comb_false = false in
  let res_comb_true = rust_combine_primitives_noalloc f_comb i_comb b_comb_true in
  Printf.printf "OCaml: rust_combine_primitives_noalloc(%f, %Ld, %b) = %Ld\n%!" f_comb i_comb b_comb_true res_comb_true;
  assert (res_comb_true = Int64.add i_comb (Int64.of_float f_comb));
  let res_comb_false = rust_combine_primitives_noalloc f_comb i_comb b_comb_false in
  Printf.printf "OCaml: rust_combine_primitives_noalloc(%f, %Ld, %b) = %Ld\n%!" f_comb i_comb b_comb_false res_comb_false;
  assert (res_comb_false = Int64.sub i_comb (Int64.of_float f_comb));

  (* Test rust_select_i64_noalloc *)
  let f_sel = 5.5 in
  let i_sel = 100L in
  let b_sel_true = true in
  let b_sel_false = false in
  let res_sel_true = rust_select_i64_noalloc f_sel i_sel b_sel_true in
  Printf.printf "OCaml: rust_select_i64_noalloc(%f, %Ld, %b) = %Ld\n%!" f_sel i_sel b_sel_true res_sel_true;
  assert (res_sel_true = Int64.add i_sel (Int64.of_float f_sel));
  let res_sel_false = rust_select_i64_noalloc f_sel i_sel b_sel_false in
  Printf.printf "OCaml: rust_select_i64_noalloc(%f, %Ld, %b) = %Ld\n%!" f_sel i_sel b_sel_false res_sel_false;
  assert (res_sel_false = Int64.sub i_sel (Int64.of_float f_sel));

  (* Test rust_log_primitives_noalloc *)
  print_endline "OCaml: calling rust_log_primitives_noalloc(1.23, 987L, true)..."; flush stdout;
  rust_log_primitives_noalloc 1.23 987L true; (* Implicitly asserts no exception *)
  print_endline "OCaml: rust_log_primitives_noalloc returned."; flush stdout;

  print_endline "--- End of noalloc examples ---"; flush stdout;
