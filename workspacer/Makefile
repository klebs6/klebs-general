.PHONY: build vendor json active test test_one test_all pin_wildcard_deps

ENTRYPOINT := example_main

#RUSTFLAGS := "-Awarnings -Z time-passes"
RUSTFLAGS  := -Awarnings RUST_BACKTRACE=1
#CARGO     := env CARGO_MSG_LIMIT=15 CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
CARGO      := MAKEFLAGS= env CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
BUILD      := build --verbose
RUN        := run
TEST       := test

#-------------------------------------[done-below]
ACTIVE_PACKAGE := command-runner
ACTIVE_PACKAGE := generate-report
ACTIVE_PACKAGE := get-file-size
ACTIVE_PACKAGE := workspacer-toml
#ACTIVE_PACKAGE := workspacer-interface
#ACTIVE_PACKAGE := workspacer-crate
#ACTIVE_PACKAGE := workspacer-3p
ACTIVE_PACKAGE := workspacer-consolidate
#ACTIVE_PACKAGE := workspacer-mock
ACTIVE_PACKAGE := workspacer-syntax
#ACTIVE_PACKAGE := workspacer

INDIVIDUAL_TEST := consolidate_crate_interface
INDIVIDUAL_TEST := test_consolidate_crate_interface_exhaustive
INDIVIDUAL_TEST := gather_impl_methods

#FEATURES := --features "enable_stress_test"
FEATURES := --features ""

TEST_FILE  := tests/test_scale_layer.rs

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
#DEFAULT := test_file
#DEFAULT := test_one
#DEFAULT := test_file_one
#DEFAULT := test_all
#DEFAULT := pin_wildcard_deps
DEFAULT := consolidate_crate_interface
#DEFAULT := name_all_files
#DEFAULT := run

default: $(DEFAULT)

#NOCAPTURE := --nocapture
#NOCAPTURE := 

#----------------------------------------------[here are our rules]

run:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) -p $(ACTIVE_PACKAGE) $(FEATURES)

pin_wildcard_dep:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-pin-wildcard-deps $(FEATURES)

consolidate_crate_interface:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-show-consolidated-crate-interface $(FEATURES) -- --path workspacer-toml --no-show-docs --no-show-private-items --no-show-fn-bodies

name_all_files:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin name-all-files $(FEATURES)

build:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(BUILD) $(FEATURES)

active:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(BUILD) -p $(ACTIVE_PACKAGE) $(FEATURES)

vendor:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) vendor

json:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(BUILD) $(FEATURES) --quiet --message-format=json 2> /dev/null | jq --slurp

#-----------------------------
test:
	RUST_LOG=trace RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) -p $(ACTIVE_PACKAGE) -- $(NOCAPTURE)

test_one:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) $(INDIVIDUAL_TEST) -p $(ACTIVE_PACKAGE) -- $(NOCAPTURE)

#-----------------------------
TEST_FILE_TARGET := $(basename $(notdir $(TEST_FILE)))

test_file:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) --test $(TEST_FILE_TARGET) -- $(NOCAPTURE)

# WARNING: make sure TEST_FILE_TARGET contains INDIVIDUAL_TEST
test_file_one:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) --test $(TEST_FILE_TARGET) $(INDIVIDUAL_TEST) -- $(NOCAPTURE)

#-----------------------------
test_all:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) -- $(NOCAPTURE)

coverage:
	cargo tarpaulin --all-features --workspace
