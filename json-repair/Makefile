default: test_one

build:
	cargo build

test: build
	cargo test

#TEST_TARGET := repair
TEST_TARGET := repair_json_add_missing_quotes
#TEST_TARGET := parse_object_tests
#TEST_TARGET := repair_json_handle_eof_between_lists_tests
#TEST_TARGET := repair_json_fix_mismatched_quotes_tests
#TEST_TARGET := repair_json_close_unexpected_eof_in_array_tag_tests
#TEST_TARGET := repair_json_control_characters
#TEST_TARGET := repair_accidental_single_quote_tests
#TEST_TARGET := repair_json_remove_duplicate_quotes_tests

NOCAPTURE := 
#NOCAPTURE := --nocapture

test_one:
	cargo test $(TEST_TARGET) -- $(NOCAPTURE)

test_repair:
	cargo test repair -- $(NOCAPTURE)
