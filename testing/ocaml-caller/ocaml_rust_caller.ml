module Rust = struct
  external twice: int -> int = "rust_twice"
  external increment_bytes: bytes -> int -> bytes = "rust_increment_bytes"
  external increment_ints_list: int list -> int list = "rust_increment_ints_list"
  external make_tuple: string -> int -> (string * int) = "rust_make_tuple"
  external make_some: string -> string option = "rust_make_some"
end

let test_twice () =
  Alcotest.(check int) "Multiply by 2" 20 (Rust.twice 10)

let test_increment_bytes () =
  let expected = Bytes.of_string "1111111111000000" in
  let result = Rust.increment_bytes (Bytes.of_string "0000000000000000") 10 in
  Alcotest.(check bytes) "Increment first 10 bytes" expected result

let test_increment_ints_list () =
  let expected = [1; 2; 3; 4; 5; 6; 7; 8; 9; 10] in
  let result = Rust.increment_ints_list [0; 1; 2; 3; 4; 5; 6; 7; 8; 9] in
  Alcotest.(check (list int)) "Increment ints in list" expected result

let test_make_tuple () =
  let expected = ("fst", 9) in
  let result = Rust.make_tuple "fst" 9 in
  Alcotest.(check (pair string int)) "Make a tuple" expected result

let test_make_some () =
  let expected = Some "some" in
  let result = Rust.make_some "some" in
  Alcotest.(check (option string)) "Make a Some(string)" expected result

let () =
  let open Alcotest in
  run "Tests" [
    "basic", [
      test_case "Rust.twice"           `Quick test_twice;
      test_case "Rust.increment_bytes" `Quick test_increment_bytes;
      test_case "Rust.increment_ints_list" `Quick test_increment_ints_list;
      test_case "Rust.make_tuple"      `Quick test_make_tuple;
      test_case "Rust.make_some"       `Quick test_make_some;
    ]
  ]