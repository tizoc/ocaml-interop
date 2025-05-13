# ocaml-interop: OCaml and Rust Integration

_Zinc-iron alloy coating is used in parts that need very good corrosion protection._

**Important Notices:**
*   **API IS CONSIDERED UNSTABLE AT THE MOMENT AND IS LIKELY TO CHANGE IN THE FUTURE.**
*   **Starting with version `0.11.0`, only OCaml 5.x is supported.**

This library facilitates interoperability between OCaml and Rust.

This document provides a structured guide to using `ocaml-interop`, covering fundamental Foreign
Function Interface (FFI) calls and progressing to more advanced concepts.

## Table of Contents

- [Part 1: Initial Usage - A Brief Overview](user_guides::part1_initial_usage_a_brief_overview)
  - 1.1 Exporting Rust Functions to OCaml: An Introduction
  - 1.2 Invoking OCaml Functions from Rust: An Introduction
  - 1.3 The OCaml Runtime Handle: `OCamlRuntime`
- [Part 2: Fundamental Concepts](user_guides::part2_fundamental_concepts)
  - 2.1 Representing OCaml Values within Rust
  - 2.2 Converting Data Between Rust and OCaml
- [Part 3: Exporting Rust Functions to OCaml](user_guides::part3_exporting_rust_functions_to_ocaml)
  - 3.1 The `#[ocaml_interop::export]` Macro
  - 3.2 Argument Types and Rooting Considerations
  - 3.3 Return Types
  - 3.4 Panic Handling Mechanisms
  - 3.5 Bytecode Function Generation
  - 3.6 The `noalloc` Attribute
  - 3.7 Direct Primitive Type Mapping
- [Part 4: Invoking OCaml Functions from Rust](user_guides::part4_invoking_ocaml_functions_from_rust)
  - 4.1 The `ocaml!` Macro
  - 4.2 Passing Arguments to OCaml
  - 4.3 Receiving Return Values from OCaml
  - 4.4 Handling OCaml Exceptions from Rust
- [Part 5: Managing the OCaml Runtime (for Rust-driven programs)](user_guides::part5_managing_the_ocaml_runtime_for_rust_driven_programs)
  - 5.1 Runtime Initialization
  - 5.2 Acquiring the Domain Lock
- [Part 6: Advanced Topics](user_guides::part6_advanced_topics)
  - 6.1 In-depth Examination of Lifetimes and GC Interaction
  - 6.2 `OCamlRef<'a, T>` Detailed Explanation
  - 6.3 Interacting with OCaml Closures
  - 6.4 Tuples
  - 6.5 Records
  - 6.6 Variants and Enums
  - 6.7 Polymorphic Variants
  - 6.8 Bigarrays (Placeholder)
  - 6.9 Threading Considerations (Placeholder)
- [Part 7: Build and Link Instructions](user_guides::part7_build_and_link_instructions)
  - Section 1: OCaml Programs Calling Rust Code
  - Section 2: Rust Programs Calling OCaml Code

### References and Further Reading

- OCaml Manual: [Chapter 20 Interfacing C with OCaml](https://v2.ocaml.org/manual/intfc.html)
- [Safely Mixing OCaml and Rust](https://docs.google.com/viewer?a=v&pid=sites&srcid=ZGVmYXVsdGRvbWFpbnxtbHdvcmtzaG9wcGV8Z3g6NDNmNDlmNTcxMDk1YTRmNg)
  paper by Stephen Dolan.
- [Safely Mixing OCaml and Rust](https://www.youtube.com/watch?v=UXfcENNM_ts) talk by Stephen Dolan.
- [CAMLroot: revisiting the OCaml FFI](https://arxiv.org/abs/1812.04905).
- [caml-oxide](https://github.com/stedolan/caml-oxide), the code from that paper.
- [ocaml-rs](https://github.com/zshipko/ocaml-rs), another OCaml<->Rust FFI library.

