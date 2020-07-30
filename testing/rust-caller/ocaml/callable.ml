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

let twice x = 2 * x

let () =
  Callback.register "increment_bytes" increment_bytes;
  Callback.register "decrement_bytes" decrement_bytes;
  Callback.register "twice" twice
