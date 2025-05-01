RUSTFLAGS := RUSTFLAGS=-Awarnings

default: test

build:
	$(RUSTFLAGS) cargo build

test: build
	$(RUSTFLAGS) cargo test

TEST_TARGET := test_repair_missing_variant_name_in_enum

NOCAPTURE := 
#NOCAPTURE := --nocapture

test_one:
	$(RUSTFLAGS) cargo test $(TEST_TARGET) -- $(NOCAPTURE)

test_repair:
	$(RUSTFLAGS) cargo test repair -- $(NOCAPTURE)
