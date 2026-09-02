DEFAULT := test

default: $(DEFAULT)

build:
	cargo build
	codesign --force --sign "osx-wallpaper-cycler" --timestamp=none target/debug/osx-wallpaper-cycler

release:
	cargo build --release
	codesign --force --sign "osx-wallpaper-cycler" --timestamp=none target/release/osx-wallpaper-cycler

test:
	cargo test

run:
	RUST_LOG=trace cargo run --release --bin osx-wallpaper-cycler run-once --config ~/.config/osx-wallpaper-cycler/config.toml

install:
	cargo install --path .

