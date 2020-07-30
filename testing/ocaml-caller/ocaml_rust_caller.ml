module Rust = struct
  external twice: int -> int = "rust_twice"
  external increment_bytes: bytes -> int -> bytes = "rust_increment_bytes"
end

let test_twice () =
  Alcotest.(check int) "Multiply by 2" 20 (Rust.twice 10)

let test_increment_bytes () =
  let expected = Bytes.of_string "1111111111000000" in
  let result = Rust.increment_bytes (Bytes.of_string "0000000000000000") 10 in
  Alcotest.(check bytes) "Increment first 10 bytes" expected result

let () =
  let open Alcotest in
  run "Tests" [
    "basic", [
      test_case "Rust.twice"           `Quick test_twice;
      test_case "Rust.increment_bytes" `Quick test_increment_bytes;
    ]
  ]