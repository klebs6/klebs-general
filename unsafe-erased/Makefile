.PHONY: build vendor json active test test_one test_all

#RUSTFLAGS := "-Awarnings -Z time-passes"
RUSTFLAGS  := -Awarnings RUST_BACKTRACE=1
#CARGO     := env CARGO_MSG_LIMIT=15 CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
CARGO      := MAKEFLAGS= env CARGO_BUILD_JOBS=12 NUM_JOBS=12 cargo 
BUILD      := build --verbose
RUN        := run
TEST       := test

FEATURES := --features "enable_stress_test"
FEATURES := --features ""

TEST_FILE  := tests/test_scale_layer.rs

#-------------------------------------[done-below]
ACTIVE_PACKAGE := unsafe-erased
#-----------------------------------------------------[]
#INDIVIDUAL_TEST := test_build_network_io_enum

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
#DEFAULT := test_file
#DEFAULT := test_one
#CEFAULT := test_file_one
DEFAULT := test_all
#DEFAULT := run

default: $(DEFAULT)

NOCAPTURE := --nocapture
#NOCAPTURE := 

#----------------------------------------------[here are our rules]

run:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) -p $(ACTIVE_PACKAGE) $(FEATURES)

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
