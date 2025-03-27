# `workspacer-toml-mock`

`workspacer-toml-mock` is a Rust library providing a fully functional mock implementation of the `CargoTomlInterface`. This mock is useful for simulating various conditions and behaviors of a `Cargo.toml` file in Rust projects, facilitating robust testing and development environments without direct file manipulation.

## Features
- **Configurable behavior**: Simulate conditions such as missing files, missing required fields, or invalid version formats to test error handling.
- **Dependency Management**: Configure and simulate dependency version updates.
- **Save Operations**: Mimic save operations with optional simulated I/O errors.
- **Bin Target and Metadata Support**: Mock bin targets, author information, license details, and more, allowing comprehensive testing.

## Usage
Integrate this crate in scenarios where you require a mock `Cargo.toml` to test behavior without interacting with the actual files. Utilize the `MockCargoToml` struct to set desired states and methods to invoke operations like checking file existence or validating integrity.

Example:

```rust
let mock = MockCargoToml::fully_valid_config();
assert!(mock.verify_integrity().is_ok());
```

## Implementation
Implements various traits to mimic actual `Cargo.toml` interactions:
- `SaveToDisk`
- `UpdateDependencyVersionRaw`
- `GatherBinTargetNames`
- And many more

## Edition
This crate is developed for the Rust 2024 edition.
