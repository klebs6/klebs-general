.PHONY: build default test trybuild

DEFAULT := test

default: $(DEFAULT)

build:
	cargo build

test:
	RUST_BACKTRACE=1 cargo test

trybuild: 
	TRYBUILD=overwrite cargo test --test compile_fail_tests
