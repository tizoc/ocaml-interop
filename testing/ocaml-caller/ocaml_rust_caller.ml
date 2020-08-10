module Rust = struct
  external twice: int -> int = "rust_twice"
  external increment_bytes: bytes -> int -> bytes = "rust_increment_bytes"
  external make_tuple: string -> int -> (string * int) = "rust_make_tuple"
end

let test_twice () =
  Alcotest.(check int) "Multiply by 2" 20 (Rust.twice 10)

let test_increment_bytes () =
  let expected = Bytes.of_string "1111111111000000" in
  let result = Rust.increment_bytes (Bytes.of_string "0000000000000000") 10 in
  Alcotest.(check bytes) "Increment first 10 bytes" expected result

let test_make_tuple () =
  let expected = ("fst", 9) in
  let result = Rust.make_tuple "fst" 9 in
  Alcotest.(check (pair string int)) "Make a tuple" expected result

let () =
  let open Alcotest in
  run "Tests" [
    "basic", [
      test_case "Rust.twice"           `Quick test_twice;
      test_case "Rust.increment_bytes" `Quick test_increment_bytes;
      test_case "Rust.make_tuple"      `Quick test_make_tuple;
    ]
  ]