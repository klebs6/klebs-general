# workspacer-consolidate

**workspacer-consolidate** is a utility crate that consolidates a Rust crate’s public interface into a single, unified structure. It scans the source files of a crate for public items—such as functions, structs, enums, traits, type aliases, and macros—and aggregates them into a comprehensive interface object. This makes it easier to analyze, document, or further process a crate’s public API.

## Features

- **Consolidated Interface Extraction:**  
  Uses asynchronous operations to read and parse source files, gathering all public API items into one consolidated interface.

- **Async Parsing:**  
  Leverages Tokio for non-blocking file access and parsing using a dedicated syntax module with support for edition 2024.

- **Extensible Design:**  
  Designed to integrate seamlessly with the rest of the Workspacer suite (such as workspacer_3p, workspacer_crate, workspacer_interface, and workspacer_syntax) while remaining focused on the interface consolidation task.

- **Display-Friendly:**  
  Implements the Display trait so that the consolidated interface can be easily formatted and printed, making it suitable for generating documentation or analysis reports.

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
workspacer-consolidate = "0.1.0"
```

## Usage

Below is a brief example of how you might use **workspacer-consolidate**:

```rust
use workspacer_consolidate::{ConsolidateCrateInterface, ConsolidatedCrateInterface};
use workspacer_crate::CrateHandle;
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assume you have a CrateHandle for the crate you want to consolidate.
    let crate_path = PathBuf::from("path/to/your/crate");
    let crate_handle = CrateHandle::new(&crate_path).await?;

    // Consolidate the crate's public interface.
    let consolidated_interface = crate_handle.consolidate_crate_interface().await?;

    // Print the consolidated interface.
    println!("{}", consolidated_interface);

    Ok(())
}
```

In this example, the crate’s public items are extracted and then printed using the Display implementation on the consolidated interface.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/klebs6/klebs-general) for details on how to contribute and report issues.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
