.PHONY: build vendor json active test test_one test_all

#RUSTFLAGS := "-Awarnings -Z time-passes"
RUSTFLAGS  := -Awarnings RUST_BACKTRACE=1
#RUSTFLAGS  := 
#CARGO     := env CARGO_MSG_LIMIT=15 CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
CARGO      := MAKEFLAGS= env CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
BUILD      := build --verbose
RUN        := run
TEST       := test

FEATURES := --features "enable_stress_test"
FEATURES := --features ""

TEST_FILE  := tests/test_scale_layer.rs

#-------------------------------------[done-below]
#ACTIVE_PACKAGE := batch-mode-3p
ACTIVE_PACKAGE := batch-mode
#ACTIVE_PACKAGE := batch-mode-batch-executor
#ACTIVE_PACKAGE := batch-mode-batch-index
#ACTIVE_PACKAGE := batch-mode-batch-metadata
#ACTIVE_PACKAGE := batch-mode-batch-schema
ACTIVE_PACKAGE := language-model-batch-workflow-derive
#ACTIVE_PACKAGE := ai-json-template-derive
#ACTIVE_PACKAGE := batch-mode-batch-workflow
#ACTIVE_PACKAGE := batch-mode-batch-client
#ACTIVE_PACKAGE := batch-mode-batch-reconciliation
#ACTIVE_PACKAGE := batch-mode-process-response
#ACTIVE_PACKAGE := batch-mode-batch-workspace
#ACTIVE_PACKAGE := batch-mode-batch-workspace-interface
#ACTIVE_PACKAGE := batch-mode-json
#ACTIVE_PACKAGE := camel-case-token-with-comment
#ACTIVE_PACKAGE := save-load-traits
#ACTIVE_PACKAGE := batch-mode-batch-triple
#ACTIVE_PACKAGE := ai-json-template
#ACTIVE_PACKAGE := ai-json-template-derive
#ACTIVE_PACKAGE := save-load-traits
#ACTIVE_PACKAGE := save-load-derive

#-----------------------------------------------------[]
#INDIVIDUAL_TEST := test_build_network_io_enum

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
#DEFAULT := test_file
#DEFAULT := test_one
#DEFAULT := test_file_one
DEFAULT := test_all
#DEFAULT := run
#DEFAULT := file_diff
#DEFAULT := build

default: $(DEFAULT)

#NOCAPTURE := --nocapture
#NOCAPTURE := 

#----------------------------------------------[here are our rules]
run:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) -p $(ACTIVE_PACKAGE) $(FEATURES)

file_diff:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) --bin camel-case-token-file-difference -- --file-a $(FILE_A) --file-b $(FILE_B) --file-c $(FILE_C)

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
