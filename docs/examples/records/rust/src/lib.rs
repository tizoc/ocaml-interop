use ocaml_interop::{FromOCaml, OCaml, OCamlInt, OCamlRuntime, ToOCaml};

// Define the plain Rust struct.
#[derive(Debug, Clone, ToOCaml, FromOCaml)]
struct Person {
    full_name: String,
    #[ocaml(as_ = "OCamlInt")]
    birth_year: i64,
    is_active: bool,
}

#[ocaml_interop::export]
pub fn rust_update_person_activity(
    cr: &mut OCamlRuntime,
    person_val: OCaml<Person>, // Input: OCaml person record, represented by OCaml<Person>
    new_activity_status: OCaml<bool>,
) -> OCaml<Person> {
    // Output: OCaml person record
    // Convert the incoming OCaml record to our Rust `Person` struct
    let mut person_rust: Person = person_val.to_rust();

    // Modify the Rust struct
    person_rust.is_active = new_activity_status.to_rust();
    person_rust.full_name = format!("{} (updated)", person_rust.full_name);

    // Convert the modified Rust struct back to an OCaml record for returning
    person_rust.to_ocaml(cr)
}

#[ocaml_interop::export]
pub fn rust_create_person(
    cr: &mut OCamlRuntime,
    full_name_val: OCaml<String>,
    birth_year_val: OCaml<OCamlInt>,
    is_active_val: OCaml<bool>,
) -> OCaml<Person> {
    // Create an instance of our Rust `Person` struct
    let person_rust = Person {
        full_name: full_name_val.to_rust(),
        birth_year: birth_year_val.to_rust(),
        is_active: is_active_val.to_rust(),
    };
    // Convert the Rust struct to an OCaml record for returning
    person_rust.to_ocaml(cr)
}
