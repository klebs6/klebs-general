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
#INDIVIDUAL_TEST := address_record_tests
#INDIVIDUAL_TEST := attempt_storing_house_number_aggregator_in_db_tests
#INDIVIDUAL_TEST := build_all_region_data_tests
#INDIVIDUAL_TEST := collect_address_and_housenumber_data_tests
#INDIVIDUAL_TEST := extract_city_from_key_tests
#INDIVIDUAL_TEST := house_number_aggregator_tests
#INDIVIDUAL_TEST := test_database_put_get
#INDIVIDUAL_TEST := test_integrate_house_number_subranges_for_street
#INDIVIDUAL_TEST := test_load_existing_street_ranges
#INDIVIDUAL_TEST := test_load_osm_data_with_housenumbers
#INDIVIDUAL_TEST := test_merge_new_subranges
#INDIVIDUAL_TEST := test_mock_node
#INDIVIDUAL_TEST := test_obtain_pbf_file_for_region
#INDIVIDUAL_TEST := test_open_pbf_reader_or_report_error
#INDIVIDUAL_TEST := test_parse_address_record_if_any
#INDIVIDUAL_TEST := test_parse_integer
#INDIVIDUAL_TEST := test_process_and_validate_addresses
#INDIVIDUAL_TEST := test_process_single_osm_element
#INDIVIDUAL_TEST := test_store_house_number_ranges
#INDIVIDUAL_TEST := test_store_merged_house_number_ranges
INDIVIDUAL_TEST := test_try_assemble_address_record

#-----------------------------------------------------[this section lets us choose how we want to test]
DEFAULT := test
DEFAULT := test_one

#DEFAULT := test_file
#DEFAULT := test_file_one
#DEFAULT := test_all
#DEFAULT := run

default: $(DEFAULT)

#NOCAPTURE := --nocapture --test-threads=1
#NOCAPTURE := 

#----------------------------------------------[here are our rules]
run:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(RUN) -p $(ACTIVE_PACKAGE) $(FEATURES)

build:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(BUILD) $(FEATURES)

build_tests:
	RUSTFLAGS=$(RUSTFLAGS) $(CARGO) $(BUILD) --tests $(FEATURES)

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
