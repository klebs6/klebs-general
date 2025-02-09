# workspacer-toml

`workspacer-toml` is a utility crate within the Workspacer ecosystem that provides functionality for parsing, validating, and manipulating `Cargo.toml` files. It offers a convenient handle (`CargoToml`) to access and verify a crate’s manifest, ensuring that required fields exist and that the version string is a valid SemVer.

## Features

- **TOML Parsing and Validation:**  
  Reads and parses a `Cargo.toml` file into a `toml::Value` and validates its contents.
  
- **Required Field Checks:**  
  Ensures that the `[package]` section contains all required fields for both integrity (e.g. `name`, `version`) and publishing (e.g. `authors`, `license`).

- **Version Validation:**  
  Uses the `semver` crate to verify that the version string follows valid SemVer conventions.

- **Ready-for-Publishing Checks:**  
  Implements the `ReadyForCargoPublish` trait to determine if a crate is properly configured for Cargo publishing.

- **Unified Error Handling:**  
  Provides a comprehensive set of error types (via `error_tree!`) for issues encountered during TOML reading, parsing, or validation.

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
workspacer-toml = "0.1.0"
```

## Usage

Below is a simple example of how to use `workspacer-toml` to create a `CargoToml` handle and validate a `Cargo.toml` file:

```rust
use workspacer_toml::{CargoToml, CargoTomlInterface};
use workspacer_interface::ReadyForCargoPublish;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Provide the path to a Cargo.toml file.
    let cargo_toml_path = "path/to/Cargo.toml";
    
    // Create a CargoToml handle from the file.
    let cargo_toml = CargoToml::new(cargo_toml_path).await?;
    
    // Validate the integrity of the Cargo.toml file.
    cargo_toml.validate_integrity()?;
    
    // Check if the Cargo.toml is ready for publishing.
    cargo_toml.ready_for_cargo_publish().await?;
    
    println!("Cargo.toml is valid and ready for publishing.");
    Ok(())
}
```

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/klebs6/klebs-general) for guidelines on contributing and reporting issues.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
