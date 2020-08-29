(* Copyright (c) SimpleStaking and Tezedge Contributors
   SPDX-License-Identifier: MIT *)

type test_record = {
  i: int;
  f: float;
  i32: int32;
  i64: int64;
  s: string;
  t: int * float;
}

let increment_bytes bytes first_n =
  let limit = (min (Bytes.length bytes) first_n) - 1 in
  for i = 0 to limit do
    let value = (Bytes.get_uint8 bytes i) + 1 in
    Bytes.set_uint8 bytes i value
  done;
  bytes

let decrement_bytes bytes first_n =
    let limit = (min (Bytes.length bytes) first_n) - 1 in
    for i = 0 to limit do
      let value = (Bytes.get_uint8 bytes i) - 1 in
      Bytes.set_uint8 bytes i value
    done;
    bytes

let increment_ints_list ints =
  List.map ((+) 1) ints

let twice x = 2 * x

let make_tuple a b = (a, b)

let make_some x = Some x

let verify_record { i; f; i32; i64; s; t = (t1, t2); } =
  i = 10 &&
  Float.equal f 5.0 &&
  Int32.equal i32 10l &&
  Int64.equal i64 10L &&
  String.equal s "string" &&
  t1 = 10 &&
  Float.equal t2 5.0

let () =
  Callback.register "increment_bytes" increment_bytes;
  Callback.register "decrement_bytes" decrement_bytes;
  Callback.register "increment_ints_list" increment_ints_list;
  Callback.register "twice" twice;
  Callback.register "make_tuple" make_tuple;
  Callback.register "make_some" make_some;
  Callback.register "verify_record" verify_record