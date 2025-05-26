use ocaml_interop::{FromOCaml, OCaml, OCamlInt, OCamlRuntime, ToOCaml};

#[derive(Debug, Clone, FromOCaml, ToOCaml)]
#[ocaml(polymorphic_variant)]
pub enum Action {
    Start,
    Stop,
    #[ocaml(tag = "Set_speed")]
    SetSpeed(#[ocaml(as_ = "OCamlInt")] i64),
}

#[ocaml_interop::export]
pub fn rust_create_action(cr: &mut OCamlRuntime, _unit: OCaml<()>) -> OCaml<Action> {
    let action_rust = Action::SetSpeed(100);
    action_rust.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_process_action(cr: &mut OCamlRuntime, action: OCaml<Action>) -> OCaml<String> {
    let rust_action: Action = action.to_rust();
    let description = match rust_action {
        Action::Start => "Rust processed: Start".to_string(),
        Action::Stop => "Rust processed: Stop".to_string(), // Added missing Stop case
        Action::SetSpeed(s) => format!("Rust processed: Set_speed to {}", s),
    };
    description.to_ocaml(cr)
}
