.PHONY: build vendor json active test test_one test_all pin_wildcard_deps

ENTRYPOINT := example_main

#RUSTFLAGS := "-Awarnings -Z time-passes"
RUSTFLAGS  := -Awarnings RUST_BACKTRACE=1
#CARGO     := env CARGO_MSG_LIMIT=15 CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
CARGO      := MAKEFLAGS= env CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
BUILD      := build --verbose
RUN        := run
TEST       := test

#ACTIVE_PACKAGE := generate-report-trait
#ACTIVE_PACKAGE := get-file-size
#ACTIVE_PACKAGE := lightweight-command-runner
#ACTIVE_PACKAGE := workspacer-analysis
#-------------------------------------[done-below]
ACTIVE_PACKAGE := workspacer-crate-interface
ACTIVE_PACKAGE := workspacer-interface
ACTIVE_PACKAGE := workspacer-toml-interface
ACTIVE_PACKAGE := workspacer-workspace-interface
ACTIVE_PACKAGE := workspacer-3p
ACTIVE_PACKAGE := workspacer-add-and-initialize-crate
ACTIVE_PACKAGE := workspacer-cleanup
ACTIVE_PACKAGE := workspacer-crate
ACTIVE_PACKAGE := workspacer-detect-circular-deps
ACTIVE_PACKAGE := workspacer-docs
ACTIVE_PACKAGE := workspacer-register
ACTIVE_PACKAGE := workspacer-errors
ACTIVE_PACKAGE := workspacer-git
ACTIVE_PACKAGE := workspacer-linting
ACTIVE_PACKAGE := workspacer-lock
ACTIVE_PACKAGE := workspacer-metadata
ACTIVE_PACKAGE := workspacer-mock
ACTIVE_PACKAGE := workspacer-publish
ACTIVE_PACKAGE := workspacer-rebuild-or-test
ACTIVE_PACKAGE := workspacer-format-imports
ACTIVE_PACKAGE := workspacer-syntax

ACTIVE_PACKAGE := workspacer-test-coverage
ACTIVE_PACKAGE := workspacer-toml
ACTIVE_PACKAGE := workspacer-workspace
#ACTIVE_PACKAGE := workspacer
ACTIVE_PACKAGE := workspacer-scan-for-prefix-groups
ACTIVE_PACKAGE := workspacer-register-internal-crate-in-prefix-group
ACTIVE_PACKAGE := workspacer
ACTIVE_PACKAGE := workspacer-pin
ACTIVE_PACKAGE := workspacer-analysis
ACTIVE_PACKAGE := workspacer-scan-for-prefix-groups
ACTIVE_PACKAGE := workspacer-publish
ACTIVE_PACKAGE := workspacer-register-internal-crate-in-prefix-group
ACTIVE_PACKAGE := workspacer-check-publish-ready
ACTIVE_PACKAGE := workspacer-add-internal-dep
ACTIVE_PACKAGE := workspacer-name-all-files
ACTIVE_PACKAGE := workspacer-watch-and-reload
#ACTIVE_PACKAGE := workspacer-add-new-crate-to-workspace
#ACTIVE_PACKAGE := workspacer-bump
#ACTIVE_PACKAGE := workspacer-check-crates-io
#ACTIVE_PACKAGE := workspacer-check-private
#ACTIVE_PACKAGE := workspacer-check-publish-ready
#ACTIVE_PACKAGE := workspacer-config
#ACTIVE_PACKAGE := workspacer-crate-mock
#ACTIVE_PACKAGE := workspacer-cratesio-mock
#ACTIVE_PACKAGE := workspacer-toml-mock
#ACTIVE_PACKAGE := workspacer-workspace-mock
#ACTIVE_PACKAGE := workspacer-publish
#ACTIVE_PACKAGE := workspacer
ACTIVE_PACKAGE := workspacer-toml-mock
ACTIVE_PACKAGE := workspacer-crate-mock
ACTIVE_PACKAGE := workspacer-publish
ACTIVE_PACKAGE := workspacer-readme-writer
ACTIVE_PACKAGE := workspacer-cli
ACTIVE_PACKAGE := workspacer-consolidate
ACTIVE_PACKAGE := workspacer-lossless-file
#ACTIVE_PACKAGE := workspacer-topo

INDIVIDUAL_TEST := gather_leading_token_comments
INDIVIDUAL_TEST := gather_leading_comment_lines
#INDIVIDUAL_TEST := gather_use_items


#FEATURES := --features "enable_stress_test"
FEATURES := --features ""

TEST_FILE  := tests/test_scale_layer.rs

CONSOLIDATE_CRATE := workspacer-toml

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
#DEFAULT := build
#DEFAULT := readme_writer
#DEFAULT := readme_writer_all
#DEFAULT := test_file
#DEFAULT := test_one
#DEFAULT := test_file_one
#DEFAULT := test_all
DEFAULT := test_all_nofail
#DEFAULT := test_individual
#DEFAULT := pin_wildcard_deps
#DEFAULT := consolidate_crate_interface
#DEFAULT := ensure
#DEFAULT := publish_public_crates_in_order
#DEFAULT := run_cratesio_mock
#DEFAULT := run_workspacer
#DEFAULT := name_all_files
#DEFAULT := run

default: $(DEFAULT)

#NOCAPTURE := --nocapture
#NOCAPTURE := 

README_WRITE_CRATE := workspacer-readme-writer
README_WRITE_CRATE := workspacer-consolidate

#----------------------------------------------[here are our rules]

run:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) -p $(ACTIVE_PACKAGE) $(FEATURES)

readme_writer:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-readme-writer -- single-crate $(README_WRITE_CRATE)

readme_writer_all:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-readme-writer -- workspace ../batch-mode

pin_wildcard_deps:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-pin-wildcard-deps $(FEATURES)

run_workspacer:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin ws $(FEATURES)

run_cratesio_mock:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-cratesio-mock $(FEATURES)

publish_public_crates_in_order:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-publish-public-crates-in-order $(FEATURES) -- --dry-run

ensure:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-register $(FEATURES)

consolidate_crate_interface:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-show-consolidated-crate-interface $(FEATURES) -- --path $(CONSOLIDATE_CRATE) --no-show-private-items #--show-test-items #--show-fn-bodies-in-tests --no-show-docs --only-show-test-items

name_all_files:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin workspacer-name-all-files $(FEATURES)

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
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) --workspace -- $(NOCAPTURE)

test_all_nofail:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(TEST) --workspace --no-fail-fast -- $(NOCAPTURE)

coverage:
	cargo tarpaulin --all-features --workspace
