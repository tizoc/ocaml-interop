.PHONY: test test-examples test-all

test:
	@echo "Running main crate tests..."
	cargo test
	@echo "Running derive crate tests..."
	cargo test -p ocaml-interop-derive
	@echo "Running OCaml interop inspect tests..."
	cargo test -p ocaml-interop-inspect

test-examples:
	@echo "Running rust-caller tests..."
	cargo test -p rust-caller
	@echo "Running ocaml-caller tests..."
	cd testing/ocaml-caller; opam exec -- dune test -f
	@echo "--- Running Documentation Examples ---"
	@echo "Running Tuples example (docs/examples/tuples)..."
	cd docs/examples/tuples; opam exec -- dune test -f
	@echo "Running Records example (docs/examples/records)..."
	cd docs/examples/records; opam exec -- dune test -f
	@echo "Running Variants example (docs/examples/variants)..."
	cd docs/examples/variants; opam exec -- dune test -f
	@echo "Running Polymorphic Variants example (docs/examples/polymorphic_variants)..."
	cd docs/examples/polymorphic_variants; opam exec -- dune test -f
	@echo "Running Noalloc Primitives example (docs/examples/noalloc_primitives)..."
	cd docs/examples/noalloc_primitives; opam exec -- dune test -f
	@echo "--- Finished Documentation Examples ---"
	@echo "Running OCaml interop inspect examples..."
	cargo test -p ocaml-interop-inspect --example basic_usage
	cd inspect/examples/inspect_runtime_example; cargo run

test-all: test test-examples
	@echo "All tests (crate and examples) completed."
