.PHONY: test default build test_one

DEFAULT := test

#TEST := tests/test_both_lib_and_main_rs
#TEST := tests/test_crate_ready_for_cargo_publish
#TEST := tests/test_invalid_cargo_toml_format
TEST := test_invalid_version_format
#TEST := test/test_missing_lib_and_main_rs
#TEST := tests/test_missing_name_in_cargo_toml
#TEST := tests/test_missing_readme
#TEST := tests/test_missing_version_in_cargo_toml
#TEST := tests/test_workspace_integrity_and_publish
#TEST := tests/test_workspace_publish_ready
#TEST := tests/test_workspace_with_multiple_errors

default: $(DEFAULT)

build:
	cargo build

test_one:
	cargo test -- --test $(TEST)

test:
	cargo test

example:
	cargo run --example basic_usage
