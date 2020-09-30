.PHONY: test-all test-examples

test-all:
	cargo test
	cd testing/rust-caller; cargo test
	cd testing/ocaml-caller; dune test

test-examples:
	cd testing/rust-caller; cargo test
	cd testing/ocaml-caller; dune test
