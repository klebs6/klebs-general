# workspacer-toml

`workspacer-toml` is a Rust crate designed to manage and validate `Cargo.toml` files efficiently. It provides an interface to manipulate, inspect, and ensure the integrity of essential fields required for Rust package publishing such as `name`, `version`, `authors`, and `license`.

## Features

- **Validation**: Ensures that all mandatory fields are present and correctly formatted according to Semantic Versioning.
- **Asynchronous Operations**: Uses `tokio` for non-blocking file operations on `Cargo.toml`, enhancing performance in concurrent environments.
- **Field Manipulation**: Easily retrieve and update fields like `version`, `license`, `authors`, and `repository` using provided trait implementations.
- **Integrity Checks**: Verifies the presence of necessary fields, ensuring the file's compliance for both integrity and publishing standards.

## Technical Insights

`workspacer-toml` employs `serde` for serialization and deserialization of TOML content, and uses `toml_edit` for precise document manipulation. This combination ensures robust parsing and serialization, minimizing errors during version control and dependency management.

### Trait Implementations

- **GetPackageAuthors**: Retrieve the package's authors.
- **CheckExistence**: Confirm the existence and validity of the `Cargo.toml` file path.
- **UpdateDependencyVersionRaw**: Update the specified dependency version within the TOML.

## License

Licensed under either of MIT License or Apache License, Version 2.0 at your option.

## Repository

For access and contributions, visit the [GitHub repository](https://github.com/klebs6/klebs-general).

## Authors

- Klebs - [tpk3.mx@gmail.com](mailto:tpk3.mx@gmail.com)

## Usage

```rust
use workspacer_toml::CargoToml;

let cargo_toml = CargoToml::new("path/to/Cargo.toml").await.expect("Valid Cargo.toml required");
cargo_toml.validate_toml().expect("Validation failed");
```