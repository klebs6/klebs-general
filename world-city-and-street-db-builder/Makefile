.PHONY: validate write build vendor json active test test_one test_all

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
ACTIVE_PACKAGE := world-city-and-street-db-builder
#-----------------------------------------------------[]
INDIVIDUAL_TEST := filenames_tests
INDIVIDUAL_TEST := address_record_tests
INDIVIDUAL_TEST := city_name
INDIVIDUAL_TEST := world_address
INDIVIDUAL_TEST := test_build_world_address_fail
INDIVIDUAL_TEST := test_parse_and_aggregate_osm_empty
INDIVIDUAL_TEST := osm_parser_tests::test_parse_osm_pbf_corrupted_file
#INDIVIDUAL_TEST := verify_md5_checksum_mismatch
INDIVIDUAL_TEST := build_city_search_prefix
INDIVIDUAL_TEST := build_world_address
INDIVIDUAL_TEST := build_world_address_if_possible
INDIVIDUAL_TEST := capture_stdout
INDIVIDUAL_TEST := chain_addresses_across_files
INDIVIDUAL_TEST := collect_address_and_housenumber_data
INDIVIDUAL_TEST := db_decoder
INDIVIDUAL_TEST := expected_filename_for_region
INDIVIDUAL_TEST := gather_city_key_value_pairs
INDIVIDUAL_TEST := get_prefix_iterator
INDIVIDUAL_TEST := handle_pbf_house_number_extractor_in_thread
INDIVIDUAL_TEST := test_handle_extractor_aggregator_store_fails

#INDIVIDUAL_TEST := cli
#INDIVIDUAL_TEST := street_name_test
#INDIVIDUAL_TEST := list_all_addresses
#INDIVIDUAL_TEST := validate_all_addresses
#INDIVIDUAL_TEST := download_and_parse_all
#INDIVIDUAL_TEST := dump

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
#DEFAULT := test_file
DEFAULT := test_one
#DEFAULT := test_file_one
#DEFAULT := test_all
#DEFAULT := run

default: $(DEFAULT)


#NOCAPTURE := --nocapture
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

validate:
	cargo run -- --just-validate

write:
	cargo run -- --write-to-storage

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
