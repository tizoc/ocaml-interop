.PHONY: test-all test-examples

test-all:
	cargo test
	cargo test -p ocaml-interop-derive
	cargo test -p rust-caller
	cd testing/ocaml-caller; opam exec -- dune test -f

test-examples:
	cargo test -p rust-caller
	cd testing/ocaml-caller; opam exec -- dune test -f
