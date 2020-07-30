static RUNTIME: std::sync::Once = std::sync::Once::new();

extern "C" {
  fn caml_main(argv: *const *const i8);
  fn caml_shutdown();
}

pub fn init() {
  RUNTIME.call_once(|| {
    let arg0 = "ocaml".as_ptr() as *const i8;
    let c_args = vec![arg0, std::ptr::null()];
    unsafe { caml_main(c_args.as_ptr()) }
  });
}

pub fn shutdown() {
  unsafe { caml_shutdown() }
}
