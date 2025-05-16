use ocaml_interop::{
    impl_from_ocaml_polymorphic_variant, impl_to_ocaml_polymorphic_variant, OCaml, OCamlInt,
    OCamlRuntime, ToOCaml,
};

#[derive(Debug, Clone)]
pub enum Action {
    Start,
    Stop,
    #[allow(non_camel_case_types)]
    Set_speed(i64),
}

// Rust Action -> OCaml `action`
impl_to_ocaml_polymorphic_variant! {
    Action {
        Action::Start,
        Action::Stop,
        Action::Set_speed(speed: OCamlInt),
    }
}

// OCaml `action` -> Rust Action
impl_from_ocaml_polymorphic_variant! {
    Action {
        Start          => Action::Start,
        Stop           => Action::Stop,
        Set_speed(speed: OCamlInt) => Action::Set_speed(speed),
    }
}

#[ocaml_interop::export]
pub fn rust_create_action(cr: &mut OCamlRuntime, _unit: OCaml<()>) -> OCaml<Action> {
    let action_rust = Action::Set_speed(100);
    action_rust.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_process_action(cr: &mut OCamlRuntime, action: OCaml<Action>) -> OCaml<String> {
    let rust_action: Action = action.to_rust();
    let description = match rust_action {
        Action::Start => "Rust processed: Start".to_string(),
        Action::Stop => "Rust processed: Stop".to_string(), // Added missing Stop case
        Action::Set_speed(s) => format!("Rust processed: Set_speed to {}", s),
    };
    description.to_ocaml(cr)
}
