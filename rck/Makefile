default: test

run:
	cargo run

test:
	cargo test

cargo_install:
	 cargo install --path .
	
move:
	 mv target/release/rck $(REPO)/dev_util/util/bin/rck

install: cargo_install move
