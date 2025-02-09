# workspacer-crate

`workspacer-crate` is a core library in the Workspacer suite that focuses on analyzing individual Rust crates. It provides functionality to:

- **Extract Public Interfaces:**  
  Wrap public items (functions, structs, enums, traits, type aliases, and macros) into a unified interface using the `CrateInterfaceItem` type.

- **Analyze Crate Metrics:**  
  Compute useful metrics such as total file size, lines of code, number of source files, and test files through asynchronous analysis (`CrateAnalysis`).

- **Unified Crate Handling:**  
  Offer a high-level `CrateHandle` to manage crate-related operations, including integrity checks (e.g. verifying the existence of essential files like `Cargo.toml`, `lib.rs`/`main.rs`, and `README.md`), fetching source files with exclusions, and more.

- **Mock Crate Configuration:**  
  Easily configure and create mock crates for testing purposes using `CrateConfig`.

This crate leverages Tokio for asynchronous file I/O, along with a suite of helper libraries from the Workspacer ecosystem to provide an efficient, non-blocking approach to analyzing Rust projects.

## Features

- **Async File Analysis:**  
  Uses Tokio's async APIs to read files, count lines, and calculate file sizes without blocking your runtime.

- **Interface Extraction:**  
  Automatically extracts and displays public API items (e.g., functions, structs, traits) for further processing or documentation.

- **Comprehensive Integrity Checks:**  
  Ensures that critical files exist (e.g., `Cargo.toml`, `lib.rs`/`main.rs`, and `README.md`) to verify crate validity.

- **Extensible & Modular:**  
  Designed to integrate seamlessly with other parts of the Workspacer suite (such as workspacer-consolidate and workspacer-interface).

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-crate = "0.1.0"
```

## Usage

Below is an example of how to use `workspacer-crate` to analyze a Rust crate:

```rust
use workspacer_crate::{CrateHandle, CrateHandleInterface, CrateAnalysis, CrateConfig};
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify the path to your crate.
    let crate_path = PathBuf::from("path/to/your/crate");

    // Initialize a CrateHandle for the given crate.
    let crate_handle = CrateHandle::new(&crate_path).await?;
    
    // Validate the integrity of the crate (checks for Cargo.toml, src/ files, README.md, etc.).
    crate_handle.validate_integrity()?;
    
    // Analyze the crate to obtain metrics such as file size, line count, etc.
    let analysis = CrateAnalysis::new(&crate_handle).await?;
    println!("Total file size: {} bytes", analysis.total_file_size());
    println!("Total lines of code: {}", analysis.total_lines_of_code());
    println!("Source files: {}", analysis.total_source_files());
    println!("Test files: {}", analysis.total_test_files());
    
    // (Optional) Use CrateConfig to create a mock crate for testing.
    let config = CrateConfig::new("my_crate")
        .with_readme()
        .with_src_files()
        .with_test_files();
    println!("Mock crate config for '{}'", config.name());
    
    Ok(())
}
```

## Contributing

Contributions to `workspacer-crate` are welcome! Please refer to the [repository](https://github.com/klebs6/klebs-general) for guidelines on contributing and reporting issues.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
